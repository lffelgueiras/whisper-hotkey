use super::PostProcessor;
use crate::error::AppError;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

static LLAMA_PIDS: Lazy<StdMutex<Vec<u32>>> = Lazy::new(|| StdMutex::new(Vec::new()));

fn register_pid(pid: u32) {
    if let Ok(mut v) = LLAMA_PIDS.lock() {
        v.push(pid);
    }
}

fn unregister_pid(pid: u32) {
    if let Ok(mut v) = LLAMA_PIDS.lock() {
        v.retain(|p| *p != pid);
    }
}

pub fn kill_all_llama_servers() {
    let pids: Vec<u32> = LLAMA_PIDS
        .lock()
        .map(|v| v.clone())
        .unwrap_or_default();
    for pid in pids {
        #[cfg(unix)]
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .status();
        }
    }
    if let Ok(mut v) = LLAMA_PIDS.lock() {
        v.clear();
    }
}

const SYSTEM_PROMPT: &str = "Sua única tarefa é formatar o texto transcrito: corrigir pontuação, acentuação e capitalização. \
REGRAS ESTRITAS:\n\
- NUNCA adicione palavras que não estejam no texto original\n\
- NUNCA remova palavras do texto original\n\
- NUNCA reescreva, parafraseie, traduza ou resuma\n\
- NUNCA adicione introduções, comentários, explicações ou observações\n\
- Mantenha exatamente as mesmas palavras, na mesma ordem\n\
- Apenas adicione vírgulas, pontos finais, acentos e letras maiúsculas onde necessário\n\
- Sua resposta deve conter SOMENTE o texto formatado, nada mais";
const MAX_NEW_TOKENS: i32 = 512;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

pub struct LlamaPostProcessor {
    model_path: PathBuf,
    inner: Mutex<Option<Server>>,
}

struct Server {
    port: u16,
    client: reqwest::Client,
    _child: Arc<ChildGuard>,
}

struct ChildGuard {
    inner: Mutex<Option<Child>>,
    pid: u32,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Ok(mut g) = self.inner.try_lock() {
            if let Some(mut c) = g.take() {
                let _ = c.start_kill();
            }
        }
        unregister_pid(self.pid);
    }
}

impl LlamaPostProcessor {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            inner: Mutex::new(None),
        }
    }

    async fn ensure_server(&self) -> Result<Server, AppError> {
        let mut guard = self.inner.lock().await;
        if let Some(s) = guard.as_ref() {
            return Ok(Server {
                port: s.port,
                client: s.client.clone(),
                _child: s._child.clone(),
            });
        }
        let bin = locate_llama_server()?;
        let port = pick_free_port().await?;
        tracing::info!("starting llama-server bin={:?} port={}", bin, port);
        let child = Command::new(&bin)
            .arg("-m")
            .arg(&self.model_path)
            .arg("--port")
            .arg(port.to_string())
            .arg("--host")
            .arg("127.0.0.1")
            .arg("-c")
            .arg("2048")
            .arg("--no-webui")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Llm(format!("spawn llama-server: {e}")))?;
        let pid = child.id().ok_or_else(|| AppError::Llm("no child pid".into()))?;
        register_pid(pid);
        let guard_child = Arc::new(ChildGuard {
            inner: Mutex::new(Some(child)),
            pid,
        });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AppError::Llm(format!("http client: {e}")))?;
        wait_health(&client, port).await?;
        let s = Server {
            port,
            client: client.clone(),
            _child: guard_child.clone(),
        };
        *guard = Some(Server {
            port,
            client,
            _child: guard_child,
        });
        Ok(s)
    }
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatReq<'a> {
    messages: Vec<ChatMessage<'a>>,
    max_tokens: i32,
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResp {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatRespMessage,
}

#[derive(Deserialize)]
struct ChatRespMessage {
    content: String,
}

#[async_trait::async_trait]
impl PostProcessor for LlamaPostProcessor {
    async fn warmup(&self) -> Result<(), AppError> {
        let server = self.ensure_server().await?;
        // Real warmup: invoke the model with a tiny prompt to force weight
        // load into RAM. Without this, the first user request pays the cold
        // load cost (several seconds for an 8B Q4 model).
        let body = ChatReq {
            messages: vec![ChatMessage {
                role: "user",
                content: "oi /no_think",
            }],
            max_tokens: 1,
            temperature: 0.0,
            stream: false,
        };
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", server.port);
        match server.client.post(&url).json(&body).send().await {
            Ok(r) if r.status().is_success() => {
                tracing::info!("llm real warmup complete");
            }
            Ok(r) => {
                tracing::warn!("llm warmup non-200: {}", r.status());
            }
            Err(e) => {
                tracing::warn!("llm warmup request failed: {e}");
            }
        }
        Ok(())
    }

    async fn refine(&self, text: &str) -> Result<String, AppError> {
        let server = self.ensure_server().await?;
        // Append `/no_think` switch for Qwen3 reasoning models. Harmless for
        // non-reasoning models (just shows up as literal text in the input,
        // but our SYSTEM_PROMPT tells the model to only output the formatted
        // text so they ignore stray tokens).
        let user_content = format!("{text} /no_think");
        let body = ChatReq {
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                ChatMessage {
                    role: "user",
                    content: &user_content,
                },
            ],
            max_tokens: MAX_NEW_TOKENS,
            temperature: 0.0,
            stream: false,
        };
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", server.port);
        let resp = server
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("chat http: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Llm(format!("chat status: {}", resp.status())));
        }
        let parsed: ChatResp = resp
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("chat json: {e}")))?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        let cleaned = strip_thinking(content.trim());
        tracing::info!("llm refined text: {:?}", cleaned);
        Ok(cleaned)
    }
}

/// Strip `<think>...</think>` blocks emitted by reasoning models like Qwen3.
/// Qwen3 wraps its chain-of-thought in those tags before producing the actual
/// answer; we want only the final answer.
fn strip_thinking(input: &str) -> String {
    let mut out = input.to_string();
    while let Some(start) = out.find("<think>") {
        if let Some(end_rel) = out[start..].find("</think>") {
            let end = start + end_rel + "</think>".len();
            out.replace_range(start..end, "");
        } else {
            break;
        }
    }
    out.trim().to_string()
}

fn locate_llama_server() -> Result<PathBuf, AppError> {
    let bin_name = if cfg!(target_os = "windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let resources = dir.join("..").join("Resources");
            candidates.push(
                resources
                    .join("binaries")
                    .join("llama-runtime")
                    .join(bin_name),
            );
            candidates.push(resources.join(bin_name));
            candidates.push(dir.join(bin_name));
        }
    }
    let dev = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join("llama-runtime")
        .join(bin_name);
    candidates.push(dev);
    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }
    for fixed in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let cand = Path::new(fixed).join(bin_name);
        if cand.exists() {
            return Ok(cand);
        }
    }
    Err(AppError::Llm(format!(
        "llama-server not found. Looked in: {:?}",
        candidates
    )))
}

async fn pick_free_port() -> Result<u16, AppError> {
    let l = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::Llm(format!("port bind: {e}")))?;
    let port = l
        .local_addr()
        .map_err(|e| AppError::Llm(format!("port addr: {e}")))?
        .port();
    drop(l);
    Ok(port)
}

async fn wait_health(client: &reqwest::Client, port: u16) -> Result<(), AppError> {
    let url = format!("http://127.0.0.1:{port}/health");
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        if let Ok(r) = client.get(&url).send().await {
            if r.status().is_success() {
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            return Err(AppError::Llm("llama-server health timeout".into()));
        }
        sleep(Duration::from_millis(300)).await;
    }
}

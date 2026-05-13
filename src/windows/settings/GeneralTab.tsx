import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useConfigStore } from "@/store/configStore";
import { HotkeyCapture } from "@/components/HotkeyCapture";
import { Row, SectionTitle } from "@/components/Row";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { OverlayPosition, Theme } from "@/ipc/generated/Config";
import type { HotkeyTrigger } from "@/ipc/generated/HotkeyTrigger";

export function GeneralTab() {
  const { config, load, update } = useConfigStore();
  const [autostart, setAutostartState] = useState<boolean>(false);
  useEffect(() => {
    void load();
    void (async () => {
      try {
        const v = await invoke<boolean>("get_autostart");
        setAutostartState(v);
      } catch {}
    })();
  }, [load]);
  if (!config) return null;

  async function toggleAutostart(v: boolean) {
    await invoke("set_autostart", { enable: v });
    setAutostartState(v);
    await update({ start_at_login: v });
  }

  return (
    <div className="max-w-2xl">
      <SectionTitle>Atalho</SectionTitle>
      <Row label="Hotkey de gravação" desc="Combinação de teclas para ativar o microfone">
        <HotkeyCapture
          value={config.hotkey}
          onChange={(v) => update({ hotkey: v })}
        />
      </Row>
      <Row label="Modo do trigger" desc="Push-to-talk ou alternar">
        <Select
          value={config.hotkey_trigger}
          onValueChange={(v) => update({ hotkey_trigger: v as HotkeyTrigger })}
        >
          <SelectTrigger className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="toggle">Toggle</SelectItem>
            <SelectItem value="push_to_talk">Push to talk</SelectItem>
          </SelectContent>
        </Select>
      </Row>

      <SectionTitle>Comportamento</SectionTitle>
      <Row label="Auto-paste" desc="Envia ⌘V automaticamente após transcrever">
        <Switch
          checked={config.auto_paste}
          onCheckedChange={(v) => update({ auto_paste: v })}
        />
      </Row>
      <Row label="Iniciar com o sistema" desc="Abre minimizado no login">
        <Switch
          checked={autostart}
          onCheckedChange={(v) => void toggleAutostart(v)}
        />
      </Row>
      <Row
        label="Som de feedback"
        desc="Toca um beep ao iniciar e finalizar a gravação"
      >
        <Switch
          checked={config.sound_feedback}
          onCheckedChange={(v) => update({ sound_feedback: v })}
        />
      </Row>

      <SectionTitle>Aparência</SectionTitle>
      <Row label="Posição do overlay" desc="Onde o indicador de gravação aparece">
        <Select
          value={config.overlay_position}
          onValueChange={(v) => update({ overlay_position: v as OverlayPosition })}
        >
          <SelectTrigger className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="top_center">Topo · centro</SelectItem>
            <SelectItem value="top_left">Topo · esquerda</SelectItem>
            <SelectItem value="top_right">Topo · direita</SelectItem>
            <SelectItem value="bottom_center">Rodapé · centro</SelectItem>
          </SelectContent>
        </Select>
      </Row>
      <Row label="Tema" desc="Frost (claro) ou Obsidian (escuro)">
        <Select
          value={config.theme}
          onValueChange={(v) => update({ theme: v as Theme })}
        >
          <SelectTrigger className="w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="system">Sistema</SelectItem>
            <SelectItem value="light">Frost (claro)</SelectItem>
            <SelectItem value="dark">Obsidian (escuro)</SelectItem>
          </SelectContent>
        </Select>
      </Row>
    </div>
  );
}

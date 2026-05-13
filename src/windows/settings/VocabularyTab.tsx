import { useConfigStore } from "@/store/configStore";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { ReplacementEditor } from "@/components/ReplacementEditor";

export function VocabularyTab() {
  const { config, update } = useConfigStore();
  const [newWord, setNewWord] = useState("");
  if (!config) return null;

  function addWord() {
    const w = newWord.trim();
    if (!w) return;
    void update({ vocabulary: [...config!.vocabulary, w] });
    setNewWord("");
  }

  function removeWord(i: number) {
    const v = config!.vocabulary.filter((_, j) => j !== i);
    void update({ vocabulary: v });
  }

  return (
    <div className="mt-4 grid gap-8 max-w-xl">
      <section>
        <h3 className="font-medium mb-2">Custom words</h3>
        <div className="flex gap-2 mb-2">
          <Input
            value={newWord}
            onChange={(e) => setNewWord(e.target.value)}
            placeholder="e.g. Ploomes"
            onKeyDown={(e) => e.key === "Enter" && addWord()}
          />
          <Button onClick={addWord}>Add</Button>
        </div>
        <div className="flex flex-wrap gap-1">
          {config.vocabulary.map((w, i) => (
            <span key={i} className="rounded bg-muted px-2 py-1 text-xs">
              {w}{" "}
              <button
                className="ml-1 opacity-60 hover:opacity-100"
                onClick={() => removeWord(i)}
              >
                ×
              </button>
            </span>
          ))}
        </div>
      </section>

      <section>
        <h3 className="font-medium mb-2">Replacement rules</h3>
        <ReplacementEditor
          rules={config.replacements}
          onChange={(rs) => void update({ replacements: rs })}
        />
      </section>
    </div>
  );
}

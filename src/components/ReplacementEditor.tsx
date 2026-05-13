import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import type { ReplacementRule } from "@/ipc/generated/ReplacementRule";

interface Props {
  rules: ReplacementRule[];
  onChange: (rs: ReplacementRule[]) => void;
}

export function ReplacementEditor({ rules, onChange }: Props) {
  function update(i: number, patch: Partial<ReplacementRule>) {
    onChange(rules.map((r, j) => (j === i ? { ...r, ...patch } : r)));
  }
  function remove(i: number) {
    onChange(rules.filter((_, j) => j !== i));
  }
  function add() {
    onChange([...rules, { from: "", to: "", regex: false }]);
  }

  return (
    <div className="grid gap-2">
      {rules.map((r, i) => (
        <div key={i} className="flex gap-2 items-center">
          <Input
            value={r.from}
            placeholder="from"
            onChange={(e) => update(i, { from: e.target.value })}
          />
          <span>→</span>
          <Input
            value={r.to}
            placeholder="to"
            onChange={(e) => update(i, { to: e.target.value })}
          />
          <label className="text-xs flex items-center gap-1">
            <Switch checked={r.regex} onCheckedChange={(v) => update(i, { regex: v })} /> regex
          </label>
          <Button variant="ghost" onClick={() => remove(i)}>
            ×
          </Button>
        </div>
      ))}
      <Button variant="outline" onClick={add}>
        + Add rule
      </Button>
    </div>
  );
}

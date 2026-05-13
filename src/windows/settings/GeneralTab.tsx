import { useEffect } from "react";
import { useConfigStore } from "@/store/configStore";
import { HotkeyCapture } from "@/components/HotkeyCapture";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { OverlayPosition, Theme } from "@/ipc/generated/Config";

export function GeneralTab() {
  const { config, load, update } = useConfigStore();
  useEffect(() => {
    void load();
  }, [load]);
  if (!config) return null;

  return (
    <div className="mt-4 grid max-w-md gap-6">
      <div className="grid gap-2">
        <Label>Hotkey</Label>
        <HotkeyCapture value={config.hotkey} onChange={(v) => update({ hotkey: v })} />
      </div>
      <div className="flex items-center justify-between">
        <Label>Auto-paste</Label>
        <Switch
          checked={config.auto_paste}
          onCheckedChange={(v) => update({ auto_paste: v })}
        />
      </div>
      <div className="grid gap-2">
        <Label>Overlay position</Label>
        <Select
          value={config.overlay_position}
          onValueChange={(v) => update({ overlay_position: v as OverlayPosition })}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="top_center">Top center</SelectItem>
            <SelectItem value="top_left">Top left</SelectItem>
            <SelectItem value="top_right">Top right</SelectItem>
            <SelectItem value="bottom_center">Bottom center</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <Label>Theme</Label>
        <Select value={config.theme} onValueChange={(v) => update({ theme: v as Theme })}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="system">System</SelectItem>
            <SelectItem value="light">Light</SelectItem>
            <SelectItem value="dark">Dark</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

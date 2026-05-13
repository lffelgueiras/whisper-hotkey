import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GeneralTab } from "./GeneralTab";
import { ModelTab } from "./ModelTab";
import { VocabularyTab } from "./VocabularyTab";
import { HistoryTab } from "@/windows/history/HistoryTab";

export function SettingsWindow() {
  return (
    <div className="flex h-screen w-screen flex-col p-6">
      <div className="glass flex flex-1 flex-col overflow-hidden">
        <header className="border-b border-border/40 px-6 py-4">
          <h1 className="text-base font-semibold tracking-tight">Settings</h1>
          <p className="mt-0.5 text-xs text-muted-foreground">
            Atalho, modelos e preferências
          </p>
        </header>

        <Tabs defaultValue="general" className="flex flex-1 flex-col overflow-hidden">
          <div className="border-b border-border/40 px-6 pt-4 pb-3">
            <TabsList>
              <TabsTrigger value="general">Geral</TabsTrigger>
              <TabsTrigger value="model">Modelos</TabsTrigger>
              <TabsTrigger value="vocabulary">Vocabulário</TabsTrigger>
              <TabsTrigger value="history">Histórico</TabsTrigger>
            </TabsList>
          </div>

          <div className="flex-1 overflow-y-auto px-6 py-4">
            <TabsContent value="general" className="mt-0">
              <GeneralTab />
            </TabsContent>
            <TabsContent value="model" className="mt-0">
              <ModelTab />
            </TabsContent>
            <TabsContent value="vocabulary" className="mt-0">
              <VocabularyTab />
            </TabsContent>
            <TabsContent value="history" className="mt-0">
              <HistoryTab />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  );
}

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GeneralTab } from "./GeneralTab";
import { ModelTab } from "./ModelTab";

export function SettingsWindow() {
  return (
    <div className="h-screen w-screen p-6">
      <h1 className="mb-4 text-lg font-semibold">Settings</h1>
      <Tabs defaultValue="general" className="h-full">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="model">Model</TabsTrigger>
          <TabsTrigger value="vocabulary" disabled>
            Vocabulary
          </TabsTrigger>
        </TabsList>
        <TabsContent value="general">
          <GeneralTab />
        </TabsContent>
        <TabsContent value="model">
          <ModelTab />
        </TabsContent>
      </Tabs>
    </div>
  );
}

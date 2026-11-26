import { mount, Text, View } from "nsh";
import { Directories } from "nsh-directories";

function captureCode(fn: () => unknown): string {
  try {
    fn();
    return "ALLOWED";
  } catch (error) {
    return (error as { code?: string }).code ?? "ERROR";
  }
}

export function DeniedCapApp() {
  const homeStatus = captureCode(() => Directories.homeDir());
  const userStatus = captureCode(() => Directories.userDirs());
  const base = Directories.baseDirs();
  const project = Directories.projectDirs("com", "Example Corp", "Denied Cap");

  return (
    <View class="flex-1 gap-2 bg-background p-4">
      <Text class="text-lg font-semibold text-foreground">nsh-directories capability check</Text>
      <Text class="text-sm text-foreground">{`homeDir: ${homeStatus}`}</Text>
      <Text class="text-sm text-foreground">{`userDirs: ${userStatus}`}</Text>
      <Text class="text-sm text-foreground">{`baseDirs.homeDir: ${String(base?.homeDir)}`}</Text>
      <Text class="text-xs text-muted-foreground">{`projectDirs.configDir: ${project?.configDir ?? "null"}`}</Text>
    </View>
  );
}

mount(() => <DeniedCapApp />);

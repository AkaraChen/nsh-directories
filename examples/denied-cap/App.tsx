import { mount, platform, Text, View } from "nsh";
import os from "os";
import { Directories } from "nsh-directories";

interface DeniedCapture {
  allowed: boolean;
  code: string | null;
  permission: string | null;
  resource: string | null;
  message: string | null;
}

function captureDenied(fn: () => unknown): DeniedCapture {
  try {
    fn();
    return {
      allowed: true,
      code: null,
      permission: null,
      resource: null,
      message: null,
    };
  } catch (error) {
    const err = error as {
      code?: unknown;
      permission?: unknown;
      resource?: unknown;
      message?: unknown;
    };
    return {
      allowed: false,
      code: typeof err?.code === "string" ? err.code : "ERROR",
      permission: typeof err?.permission === "string" ? err.permission : null,
      resource: typeof err?.resource === "string" ? err.resource : null,
      message: typeof err?.message === "string" ? err.message : String(error),
    };
  }
}

export function DeniedCapApp() {
  const homeCapture = captureDenied(() => Directories.homeDir());
  const userCapture = captureDenied(() => Directories.userDirs());
  const homeStatus = homeCapture.allowed ? "ALLOWED" : (homeCapture.code ?? "ERROR");
  const userStatus = userCapture.allowed ? "ALLOWED" : (userCapture.code ?? "ERROR");
  const base = Directories.baseDirs();
  const project = Directories.projectDirs("com", "Example Corp", "Denied Cap");
  const fromPath = Directories.projectDirsFromPath("custom-app");
  const runtimePlatform = platform.os === "web" || os.arch() === "wasm32" ? "web" : os.platform();
  const snapshot = {
    platform: runtimePlatform,
    platformVersion: platform.version,
    available: Directories.available(),
    info: Directories.info(),
    homeDir: homeCapture,
    userDirs: userCapture,
    baseDirs: base,
    projectDirs: project,
    projectDirsFromPath: fromPath,
  };

  return (
    <View class="flex-1 gap-2 bg-background p-4">
      <Text class="text-lg font-semibold text-foreground">nsh-directories capability check</Text>
      <Text class="text-sm text-foreground">{`homeDir: ${homeStatus}`}</Text>
      <Text class="text-sm text-foreground">{`userDirs: ${userStatus}`}</Text>
      <Text class="text-sm text-foreground">{`baseDirs.homeDir: ${String(base?.homeDir)}`}</Text>
      <Text class="text-xs text-muted-foreground">{`projectDirs.configDir: ${project?.configDir ?? "null"}`}</Text>
      <Text testID="e2e-snapshot" class="text-xs text-muted-foreground">
        {`E2E_SNAPSHOT:${JSON.stringify(snapshot)}`}
      </Text>
    </View>
  );
}

mount(() => <DeniedCapApp />);

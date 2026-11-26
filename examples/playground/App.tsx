import { mount, platform, Scroll, Text, View } from "nsh";
import { Directories } from "nsh-directories";

type Capture<T> =
  | { ok: true; value: T }
  | { ok: false; code: string | null; message: string };

function capture<T>(fn: () => T): Capture<T> {
  try {
    return { ok: true, value: fn() };
  } catch (error) {
    const err = error as { code?: unknown; message?: unknown };
    return {
      ok: false,
      code: typeof err?.code === "string" ? err.code : null,
      message: typeof err?.message === "string" ? err.message : String(error),
    };
  }
}

export function Playground() {
  const available = Directories.available();
  const info = Directories.info();
  const homeDir = capture(() => Directories.homeDir());
  const base = Directories.baseDirs();
  const userCapture = capture(() => Directories.userDirs());
  const project = Directories.projectDirs("com", "Example Corp", "Playground");
  const projectDirsFromPath = Directories.projectDirsFromPath("custom-app");
  const snapshot = {
    platform: platform.os,
    platformVersion: platform.version,
    available,
    info,
    homeDir,
    baseDirs: base,
    userDirs: userCapture,
    projectDirs: project,
    projectDirsFromPath,
  };
  const user = userCapture.ok ? userCapture.value : userCapture;

  return (
    <Scroll class="flex-1 bg-background p-4">
      <View class="gap-2">
        <Text class="text-lg font-semibold text-foreground">nsh-directories playground</Text>
        <Text class="text-xs text-muted-foreground">
          {`available: ${available} · crate ${info.version} · upstream ${info.upstreamVersion} (${info.upstreamCommit.slice(0, 8)})`}
        </Text>
        <Text testID="e2e-snapshot" class="text-xs text-muted-foreground">
          {`E2E_SNAPSHOT:${JSON.stringify(snapshot)}`}
        </Text>
        <Text class="text-sm font-medium text-foreground">BaseDirs</Text>
        <Text class="text-xs text-muted-foreground">{JSON.stringify(base, null, 2)}</Text>
        <Text class="text-sm font-medium text-foreground">UserDirs</Text>
        <Text class="text-xs text-muted-foreground">{JSON.stringify(user, null, 2)}</Text>
        <Text class="text-sm font-medium text-foreground">ProjectDirs</Text>
        <Text class="text-xs text-muted-foreground">{JSON.stringify(project, null, 2)}</Text>
      </View>
    </Scroll>
  );
}

mount(() => <Playground />);

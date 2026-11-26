import { mount, Scroll, Text, View } from "nsh";
import { Directories } from "nsh-directories";

export function Playground() {
  const info = Directories.info();
  const base = Directories.baseDirs();
  const user = Directories.userDirs();
  const project = Directories.projectDirs("com", "Example Corp", "Playground");

  return (
    <Scroll class="flex-1 bg-background p-4">
      <View class="gap-2">
        <Text class="text-lg font-semibold text-foreground">nsh-directories playground</Text>
        <Text class="text-xs text-muted-foreground">
          {`available: ${Directories.available()} · crate ${info.version} · upstream ${info.upstreamVersion} (${info.upstreamCommit.slice(0, 8)})`}
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

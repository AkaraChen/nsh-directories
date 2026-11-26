import { expect, open, test } from "nsh/test";
import { Directories } from "nsh-directories";
import os from "os";

test("Directories.available and Directories.info report crate and upstream provenance", () => {
  expect(Directories.available()).toBe(true);
  const info = Directories.info();
  expect(info.version).toBe("0.1.0");
  expect(info.upstreamVersion).toBe("6.0.0");
  expect(info.upstreamCommit).toBe("1f285d9114176e7be8b85b354e3bfde7c3e6356e");
  expect(info.available).toBe(true);
});

test("Directories returns plain path snapshots when homeDir is granted by default", () => {
  const home = Directories.homeDir();
  expect(home).toBe(os.homedir());

  const base = Directories.baseDirs();
  expect(base).not.toBeNull();
  const spreadBase = { ...base! };
  const roundTripBase = JSON.parse(JSON.stringify(base));
  expect(roundTripBase).toEqual(spreadBase);
  expect(base!.homeDir).toBe(os.homedir());
  expect(base!.cacheDir.length > 0).toBe(true);
  expect(base!.configDir.length > 0).toBe(true);
  expect(base!.configLocalDir.length > 0).toBe(true);
  expect(base!.dataDir.length > 0).toBe(true);
  expect(base!.dataLocalDir.length > 0).toBe(true);
  expect(base!.preferenceDir.length > 0).toBe(true);

  const user = Directories.userDirs();
  expect(user).not.toBeNull();
  expect(user!.homeDir).toBe(os.homedir());

  const project = Directories.projectDirs("com", "Example Corp", "Playground");
  expect(project).not.toBeNull();
  expect(project!.projectPath.length > 0).toBe(true);
  expect(project!.configDir.length > 0).toBe(true);
  expect(project!.cacheDir.length > 0).toBe(true);

  const fromPath = Directories.projectDirsFromPath("custom-app");
  expect(fromPath).not.toBeNull();
  expect(fromPath!.projectPath).toBe("custom-app");

  // Does not alter NSH's built-in os / process modules.
  expect(typeof os.homedir()).toBe("string");
  expect(process.dataDir === null || typeof process.dataDir === "string").toBe(true);
});

test("the playground app renders the directory snapshots", async () => {
  const app = await open("./App.tsx", { platform: "linux", width: 900, height: 700 });
  await app.waitFor("text:nsh-directories playground");
  await app.waitUntil(() => app.texts().some((t) => t.startsWith("available: true")));
});

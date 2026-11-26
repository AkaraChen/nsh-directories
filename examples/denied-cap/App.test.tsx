import { expect, open, test } from "nsh/test";
import { Directories } from "nsh-directories";

interface AccessDeniedError {
  code?: string;
  permission?: string;
  resource?: string;
  message?: string;
}

function expectHomeDirDenied(fn: () => unknown) {
  let caught: AccessDeniedError | undefined;
  try {
    fn();
  } catch (error) {
    caught = error as AccessDeniedError;
  }
  expect(caught).toBeDefined();
  expect(caught!.code).toBe("ERR_ACCESS_DENIED");
  expect(caught!.permission).toBe("homeDir");
  expect(caught!.resource).toBe("");
  expect((caught!.message ?? "").includes('"homeDir": true')).toBe(true);
}

test("homeDir() and userDirs() reject with ERR_ACCESS_DENIED when capabilities.homeDir is omitted", () => {
  expectHomeDirDenied(() => Directories.homeDir());
  expectHomeDirDenied(() => Directories.userDirs());
});

test("ungated baseDirs, projectDirs, available and info succeed while baseDirs.homeDir is null", () => {
  expect(Directories.available()).toBe(true);
  const info = Directories.info();
  expect(info.version).toBe("0.1.0");
  expect(info.upstreamVersion).toBe("6.0.0");

  const base = Directories.baseDirs();
  expect(base).not.toBeNull();
  expect(base!.homeDir).toBeNull();
  expect(base!.cacheDir.length > 0).toBe(true);
  expect(base!.configDir.length > 0).toBe(true);
  expect(base!.configLocalDir.length > 0).toBe(true);
  expect(base!.dataDir.length > 0).toBe(true);
  expect(base!.dataLocalDir.length > 0).toBe(true);
  expect(base!.preferenceDir.length > 0).toBe(true);

  const project = Directories.projectDirs("com", "Example Corp", "Denied Cap");
  expect(project).not.toBeNull();
  expect(project!.configDir.length > 0).toBe(true);
  expect(project!.cacheDir.length > 0).toBe(true);

  const fromPath = Directories.projectDirsFromPath("custom-app");
  expect(fromPath).not.toBeNull();
  expect(fromPath!.projectPath).toBe("custom-app");
});

test("the denied-cap app renders expected capability enforcement", async () => {
  const app = await open("./App.tsx", { platform: "linux", width: 800, height: 600 });
  await app.waitFor("text:nsh-directories capability check");
  await app.waitFor("text:homeDir: ERR_ACCESS_DENIED");
  await app.waitFor("text:userDirs: ERR_ACCESS_DENIED");
  await app.waitFor("text:baseDirs.homeDir: null");
});

import { expect, open, test } from "nsh/test";

interface DirectoriesInfoSnapshot {
  version: string;
  upstreamVersion: string;
  upstreamCommit: string;
  available: boolean;
}

interface DeniedCapture {
  allowed: boolean;
  code: string | null;
  permission: string | null;
  resource: string | null;
  message: string | null;
}

interface BaseDirsSnapshot {
  homeDir: string | null;
  cacheDir: string;
  configDir: string;
  configLocalDir: string;
  dataDir: string;
  dataLocalDir: string;
  executableDir: string | null;
  preferenceDir: string;
  runtimeDir: string | null;
  stateDir: string | null;
}

interface ProjectDirsSnapshot {
  projectPath: string;
  cacheDir: string;
  configDir: string;
  configLocalDir: string;
  dataDir: string;
  dataLocalDir: string;
  preferenceDir: string;
  runtimeDir: string | null;
  stateDir: string | null;
}

interface DeniedCapSnapshot {
  platform: string;
  platformVersion: string;
  available: boolean;
  info: DirectoriesInfoSnapshot;
  homeDir: DeniedCapture;
  userDirs: DeniedCapture;
  baseDirs: BaseDirsSnapshot | null;
  projectDirs: ProjectDirsSnapshot | null;
  projectDirsFromPath: ProjectDirsSnapshot | null;
}

const SNAPSHOT_PREFIX = "E2E_SNAPSHOT:";

function expectHomeDirDenied(capture: DeniedCapture) {
  expect(capture.allowed).toBe(false);
  expect(capture.code).toBe("ERR_ACCESS_DENIED");
  expect(capture.permission).toBe("homeDir");
  expect(capture.resource).toBe("");
  expect((capture.message ?? "").includes('"homeDir": true')).toBe(true);
}

test("renders and verifies capability gates on real target platforms", async () => {
  const app = await open("./App.tsx");
  await app.waitFor("id:e2e-snapshot");
  await app.waitUntil(() => app.texts().some((t) => t.startsWith(SNAPSHOT_PREFIX)));

  const raw = app.texts().find((t) => t.startsWith(SNAPSHOT_PREFIX));
  expect(raw).toBeDefined();
  const snapshot = JSON.parse(raw!.slice(SNAPSHOT_PREFIX.length)) as DeniedCapSnapshot;

  expect(snapshot.info.version).toBe("0.1.0");
  expect(snapshot.info.upstreamVersion).toBe("6.0.0");
  expect(snapshot.info.upstreamCommit).toBe("1f285d9114176e7be8b85b354e3bfde7c3e6356e");
  expect(snapshot.info.available).toBe(snapshot.available);

  expectHomeDirDenied(snapshot.homeDir);
  expectHomeDirDenied(snapshot.userDirs);

  if (snapshot.platform === "web") {
    expect(snapshot.available).toBe(false);
    expect(snapshot.baseDirs).toBeNull();
    expect(snapshot.projectDirs).toBeNull();
    expect(snapshot.projectDirsFromPath).toBeNull();
  } else if (snapshot.platform === "ios") {
    expect(snapshot.available).toBe(true);
    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBeNull();
    expect(base!.cacheDir.includes("/Containers/Data/Application/")).toBe(true);
    expect(base!.cacheDir.endsWith("/Library/Caches")).toBe(true);
    expect(base!.configDir.endsWith("/Library/Application Support")).toBe(true);
    expect(base!.dataDir.endsWith("/Library/Application Support")).toBe(true);
    expect(base!.preferenceDir.endsWith("/Library/Preferences")).toBe(true);

    const project = snapshot.projectDirs;
    expect(project).not.toBeNull();
    expect(project!.projectPath).toBe("com.Example-Corp.Denied-Cap");
    expect(project!.configDir.endsWith("/Library/Application Support/com.Example-Corp.Denied-Cap")).toBe(true);
    expect(snapshot.projectDirsFromPath?.projectPath).toBe("custom-app");
  } else if (snapshot.platform === "android") {
    expect(snapshot.available).toBe(true);
    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBeNull();
    expect(/^\/data\/user\/\d+\/[^/]+\/cache$/.test(base!.cacheDir)).toBe(true);
    expect(/^\/data\/user\/\d+\/[^/]+\/files$/.test(base!.configDir)).toBe(true);
    expect(/^\/data\/user\/\d+\/[^/]+\/no_backup$/.test(base!.configLocalDir)).toBe(true);
    expect(base!.dataDir).toBe(base!.configDir);
    expect(base!.dataLocalDir).toBe(base!.configLocalDir);
    expect(base!.preferenceDir).toBe(base!.configDir);

    const project = snapshot.projectDirs;
    expect(project).not.toBeNull();
    expect(project!.projectPath).toBe("com.Example-Corp.Denied-Cap");
    expect(project!.configDir).toBe(`${base!.configDir}/com.Example-Corp.Denied-Cap`);
    expect(project!.configLocalDir).toBe(`${base!.configLocalDir}/com.Example-Corp.Denied-Cap`);
    expect(project!.cacheDir).toBe(`${base!.cacheDir}/com.Example-Corp.Denied-Cap`);
    expect(snapshot.projectDirsFromPath?.projectPath).toBe("custom-app");
  } else if (
    snapshot.platform === "linux" ||
    snapshot.platform === "macos" ||
    snapshot.platform === "windows"
  ) {
    expect(snapshot.available).toBe(true);
    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBeNull();
    expect(base!.cacheDir.length > 0).toBe(true);
    expect(base!.configDir.length > 0).toBe(true);
    expect(base!.dataDir.length > 0).toBe(true);
    expect(base!.preferenceDir.length > 0).toBe(true);
    expect(snapshot.projectDirs).not.toBeNull();
    expect(snapshot.projectDirsFromPath?.projectPath).toBe("custom-app");
  } else {
    throw new Error(`unexpected platform in E2E snapshot: ${snapshot.platform}`);
  }

  const summary = {
    available: snapshot.available,
    homeDirCode: snapshot.homeDir.code,
    userDirsCode: snapshot.userDirs.code,
    baseHomeDir: snapshot.baseDirs?.homeDir ?? null,
    baseConfigDir: snapshot.baseDirs?.configDir ?? null,
    projectConfigDir: snapshot.projectDirs?.configDir ?? null,
  };
  console.log(
    `[e2e] platform=${snapshot.platform} version=${snapshot.platformVersion} summary=${JSON.stringify(summary)}`,
  );
});

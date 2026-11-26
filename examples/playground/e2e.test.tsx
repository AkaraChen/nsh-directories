import { expect, open, test } from "nsh/test";

interface DirectoriesInfoSnapshot {
  version: string;
  upstreamVersion: string;
  upstreamCommit: string;
  available: boolean;
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

interface UserDirsSnapshot {
  homeDir: string;
  audioDir: string | null;
  desktopDir: string | null;
  documentDir: string | null;
  downloadDir: string | null;
  fontDir: string | null;
  pictureDir: string | null;
  publicDir: string | null;
  templateDir: string | null;
  videoDir: string | null;
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

type Capture<T> =
  | { ok: true; value: T }
  | { ok: false; code: string | null; message: string };

interface PlaygroundSnapshot {
  platform: string;
  platformVersion: string;
  available: boolean;
  info: DirectoriesInfoSnapshot;
  homeDir: Capture<string | null>;
  baseDirs: BaseDirsSnapshot | null;
  userDirs: Capture<UserDirsSnapshot | null>;
  projectDirs: ProjectDirsSnapshot | null;
  projectDirsFromPath: ProjectDirsSnapshot | null;
}

const SNAPSHOT_PREFIX = "E2E_SNAPSHOT:";

test("renders and verifies real-platform Directories results", async () => {
  const app = await open("./App.tsx");
  await app.waitFor("id:e2e-snapshot");
  await app.waitUntil(() => app.texts().some((t) => t.startsWith(SNAPSHOT_PREFIX)));

  const raw = app.texts().find((t) => t.startsWith(SNAPSHOT_PREFIX));
  expect(raw).toBeDefined();
  const snapshot = JSON.parse(raw!.slice(SNAPSHOT_PREFIX.length)) as PlaygroundSnapshot;

  expect(snapshot.info.version).toBe("0.1.0");
  expect(snapshot.info.upstreamVersion).toBe("6.0.0");
  expect(snapshot.info.upstreamCommit).toBe("1f285d9114176e7be8b85b354e3bfde7c3e6356e");
  expect(snapshot.info.available).toBe(snapshot.available);

  if (snapshot.platform === "web") {
    expect(snapshot.available).toBe(false);
    expect(snapshot.info.available).toBe(false);
    expect(snapshot.homeDir).toEqual({ ok: true, value: null });
    expect(snapshot.baseDirs).toBeNull();
    expect(snapshot.userDirs).toEqual({ ok: true, value: null });
    expect(snapshot.projectDirs).toBeNull();
    expect(snapshot.projectDirsFromPath).toBeNull();
  } else if (snapshot.platform === "ios") {
    expect(snapshot.available).toBe(true);
    expect(snapshot.info.available).toBe(true);
    expect(snapshot.homeDir.ok).toBe(true);
    const home = snapshot.homeDir.ok ? snapshot.homeDir.value : null;
    expect(typeof home).toBe("string");
    expect(home!.includes("/Containers/Data/Application/")).toBe(true);

    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBe(home);
    expect(base!.cacheDir).toBe(`${home}/Library/Caches`);
    expect(base!.configDir).toBe(`${home}/Library/Application Support`);
    expect(base!.configLocalDir).toBe(`${home}/Library/Application Support`);
    expect(base!.dataDir).toBe(`${home}/Library/Application Support`);
    expect(base!.dataLocalDir).toBe(`${home}/Library/Application Support`);
    expect(base!.preferenceDir).toBe(`${home}/Library/Preferences`);
    expect(base!.executableDir).toBeNull();
    expect(base!.runtimeDir).toBeNull();
    expect(base!.stateDir).toBeNull();

    expect(snapshot.userDirs.ok).toBe(true);
    const user = snapshot.userDirs.ok ? snapshot.userDirs.value : null;
    expect(user).not.toBeNull();
    expect(user!.homeDir).toBe(home);
    expect(user!.documentDir).toBe(`${home}/Documents`);
    expect(user!.audioDir).toBeNull();
    expect(user!.desktopDir).toBeNull();
    expect(user!.downloadDir).toBeNull();
    expect(user!.fontDir).toBeNull();
    expect(user!.pictureDir).toBeNull();
    expect(user!.publicDir).toBeNull();
    expect(user!.templateDir).toBeNull();
    expect(user!.videoDir).toBeNull();

    const project = snapshot.projectDirs;
    expect(project).not.toBeNull();
    expect(project!.projectPath).toBe("com.Example-Corp.Playground");
    expect(project!.cacheDir).toBe(`${home}/Library/Caches/com.Example-Corp.Playground`);
    expect(project!.configDir).toBe(`${home}/Library/Application Support/com.Example-Corp.Playground`);
    expect(project!.configLocalDir).toBe(`${home}/Library/Application Support/com.Example-Corp.Playground`);
    expect(project!.dataDir).toBe(`${home}/Library/Application Support/com.Example-Corp.Playground`);
    expect(project!.dataLocalDir).toBe(`${home}/Library/Application Support/com.Example-Corp.Playground`);
    expect(project!.preferenceDir).toBe(`${home}/Library/Preferences/com.Example-Corp.Playground`);
    expect(project!.runtimeDir).toBeNull();
    expect(project!.stateDir).toBeNull();

    const fromPath = snapshot.projectDirsFromPath;
    expect(fromPath).not.toBeNull();
    expect(fromPath!.projectPath).toBe("custom-app");
    expect(fromPath!.cacheDir).toBe(`${home}/Library/Caches/custom-app`);
    expect(fromPath!.configDir).toBe(`${home}/Library/Application Support/custom-app`);
  } else if (snapshot.platform === "android") {
    expect(snapshot.available).toBe(true);
    expect(snapshot.info.available).toBe(true);
    expect(snapshot.homeDir.ok).toBe(true);
    const home = snapshot.homeDir.ok ? snapshot.homeDir.value : null;
    expect(typeof home).toBe("string");
    expect(home).not.toBe("/");
    expect(/^\/data\/user\/\d+\/[^/]+$/.test(home!)).toBe(true);

    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBe(home);
    expect(base!.cacheDir).toBe(`${home}/cache`);
    expect(base!.configDir).toBe(`${home}/files`);
    expect(base!.configDir).not.toBe("/.config");
    expect(base!.dataDir).toBe(`${home}/files`);
    expect(base!.preferenceDir).toBe(`${home}/files`);
    expect(base!.configLocalDir).toBe(`${home}/no_backup`);
    expect(base!.dataLocalDir).toBe(`${home}/no_backup`);
    expect(base!.executableDir).toBeNull();
    expect(base!.runtimeDir).toBeNull();
    expect(base!.stateDir).toBeNull();

    expect(snapshot.userDirs).toEqual({ ok: true, value: null });

    const project = snapshot.projectDirs;
    expect(project).not.toBeNull();
    expect(project!.projectPath).toBe("com.Example-Corp.Playground");
    expect(project!.cacheDir).toBe(`${home}/cache/com.Example-Corp.Playground`);
    expect(project!.configDir).toBe(`${home}/files/com.Example-Corp.Playground`);
    expect(project!.configLocalDir).toBe(`${home}/no_backup/com.Example-Corp.Playground`);
    expect(project!.dataDir).toBe(`${home}/files/com.Example-Corp.Playground`);
    expect(project!.dataLocalDir).toBe(`${home}/no_backup/com.Example-Corp.Playground`);
    expect(project!.preferenceDir).toBe(`${home}/files/com.Example-Corp.Playground`);
    expect(project!.runtimeDir).toBeNull();
    expect(project!.stateDir).toBeNull();

    const fromPath = snapshot.projectDirsFromPath;
    expect(fromPath).not.toBeNull();
    expect(fromPath!.projectPath).toBe("custom-app");
    expect(fromPath!.cacheDir).toBe(`${home}/cache/custom-app`);
    expect(fromPath!.configDir).toBe(`${home}/files/custom-app`);
    expect(fromPath!.configLocalDir).toBe(`${home}/no_backup/custom-app`);
  } else if (
    snapshot.platform === "linux" ||
    snapshot.platform === "macos" ||
    snapshot.platform === "windows"
  ) {
    expect(snapshot.available).toBe(true);
    expect(snapshot.homeDir.ok).toBe(true);
    const home = snapshot.homeDir.ok ? snapshot.homeDir.value : null;
    expect(typeof home).toBe("string");
    expect(home!.length > 0).toBe(true);

    const base = snapshot.baseDirs;
    expect(base).not.toBeNull();
    expect(base!.homeDir).toBe(home);
    expect(base!.cacheDir.length > 0).toBe(true);
    expect(base!.configDir.length > 0).toBe(true);
    expect(base!.dataDir.length > 0).toBe(true);
    expect(base!.preferenceDir.length > 0).toBe(true);

    expect(snapshot.userDirs.ok).toBe(true);
    const user = snapshot.userDirs.ok ? snapshot.userDirs.value : null;
    expect(user).not.toBeNull();
    expect(user!.homeDir).toBe(home);

    expect(snapshot.projectDirs).not.toBeNull();
    expect(snapshot.projectDirsFromPath?.projectPath).toBe("custom-app");
  } else {
    throw new Error(`unexpected platform in E2E snapshot: ${snapshot.platform}`);
  }

  const summary = {
    available: snapshot.available,
    homeDir: snapshot.homeDir.ok ? snapshot.homeDir.value : snapshot.homeDir.code,
    baseCacheDir: snapshot.baseDirs?.cacheDir ?? null,
    baseConfigDir: snapshot.baseDirs?.configDir ?? null,
    baseConfigLocalDir: snapshot.baseDirs?.configLocalDir ?? null,
    userDocumentDir: snapshot.userDirs.ok ? (snapshot.userDirs.value?.documentDir ?? null) : null,
    userDirsIsNull: snapshot.userDirs.ok ? snapshot.userDirs.value === null : false,
    projectConfigDir: snapshot.projectDirs?.configDir ?? null,
    fromPathConfigDir: snapshot.projectDirsFromPath?.configDir ?? null,
  };
  console.log(
    `[e2e] platform=${snapshot.platform} version=${snapshot.platformVersion} summary=${JSON.stringify(summary)}`,
  );
});

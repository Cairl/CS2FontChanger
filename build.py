#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""CS2 Font Changer: build then deploy.

参考 WorkBuddySwitcher/build.py 的职责链设计，针对本项目（CLI 工具、单文件 exe）做了适配：
1. 修正环境变量，把 USERPROFILE / RUSTUP_HOME / CARGO_HOME 钉死到真实主目录。
   父进程有时会把 USERPROFILE 重定向到奇怪的位置（如 S:\\c\\Users\\Administrator），
   rustup 代理默认按 USERPROFILE 找工具链，会在错误路径下创建整套家目录。
2. 查找可用的 cargo：优先 PATH 里的 rustup 代理；代理失效时回退到
   真实主目录 .rustup\\toolchains 下的工具链，直连编译。
3. cargo build --release。
4. 把 target\\release\\cs2_font_changer.exe 复制到项目根目录，作为最终分发产物。
   若根目录的旧实例正占用该文件，自动结束它（只结束 exe 路径完全一致的进程）。
5. 清理杂物间：只删 target 中未被当前构建引用的旧产物（按 .fingerprint 精确判定）。
6. 可选：--launch 启动构建好的 exe；--build-only 仅构建不部署。

用法：
    python build.py                # 构建并部署到项目根目录
    python build.py --launch       # 构建、部署并立即启动
    python build.py --build-only   # 只编译，不部署、不结束旧实例
"""
import os
import sys
import time
import shutil
import subprocess
from pathlib import Path

PROJ = Path(__file__).resolve().parent
EXE_NAME = "cs2_font_changer.exe"
BUILD_ONLY = "--build-only" in sys.argv
LAUNCH = "--launch" in sys.argv


def real_home() -> Path:
    """真实主目录：优先 HOMEDRIVE + HOMEPATH，否则取用户主目录。"""
    home_drive = os.environ.get("HOMEDRIVE")
    home_path = os.environ.get("HOMEPATH")
    if home_drive and home_path:
        return Path(home_drive + home_path)
    return Path.home()


def pin_env() -> dict:
    """把 USERPROFILE / RUSTUP_HOME / CARGO_HOME 钉死到真实主目录，返回环境副本。"""
    home = real_home()
    env = os.environ.copy()
    env["USERPROFILE"] = str(home)
    env["RUSTUP_HOME"] = str(home / ".rustup")
    env["CARGO_HOME"] = str(home / ".cargo")
    return env


def find_cargo(env: dict):
    """找到可用的 cargo：优先 PATH 里的 rustup 代理，失效则回退到工具链直连。"""
    cargo = shutil.which("cargo")
    if cargo:
        try:
            result = subprocess.run(
                [cargo, "--version"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                env=env,
            )
            if result.returncode == 0:
                return cargo
        except OSError:
            pass

    # 代理失效，回退到真实主目录下的工具链，并把工具链 bin 放到 PATH 最前
    tc_root = real_home() / ".rustup" / "toolchains"
    if tc_root.is_dir():
        for toolchain in sorted(tc_root.iterdir()):
            if not toolchain.is_dir():
                continue
            candidate = toolchain / "bin" / "cargo.exe"
            if candidate.is_file():
                env["PATH"] = str(toolchain / "bin") + os.pathsep + env.get("PATH", "")
                env["RUSTUP_HOME"] = str(real_home() / ".rustup")
                env["CARGO_HOME"] = str(real_home() / ".cargo")
                print(f"cargo 代理失效，回退到工具链直连：{candidate}")
                return str(candidate)
    return None


def clear_stale_deps(target_root: Path, mode: str) -> None:
    """按构建记录精确清理 deps 里的旧产物。

    cargo 每次构建成功后，会在 .fingerprint\\<crate>-<哈希>\\ 留下记录，
    deps 里被当前构建引用的产物文件名都带这些哈希。文件名哈希对不上任何
    fingerprint 的，就是历史遗留的旧产物，可安全删除。
    """
    fp_dir = target_root / mode / ".fingerprint"
    deps_dir = target_root / mode / "deps"
    if not fp_dir.is_dir() or not deps_dir.is_dir():
        print(f"跳过 {mode}：缺少构建记录或产物目录")
        return

    hashes = []
    for entry in fp_dir.iterdir():
        if entry.is_dir():
            idx = entry.name.rfind("-")
            if idx != -1:
                hashes.append(entry.name[idx + 1 :])
    if not hashes:
        print(f"跳过 {mode}：无构建记录")
        return

    stale = [
        f
        for f in deps_dir.iterdir()
        if f.is_file() and not any(h in f.name for h in hashes)
    ]
    for f in stale:
        try:
            f.unlink()
        except OSError:
            pass
    print(f"已清理 {mode} 旧产物 {len(stale)} 个，当前构建引用的缓存全部保留")


def break_deploy_hardlink(src: Path, dst: Path) -> bool:
    """把「根目录 exe 与编译产物是同一个文件」这一态改成两个独立文件。

    背景：cargo 的 uplift 把 `target\\release\\X.exe` 硬链接到
    `target\\release\\deps\\X-<hash>.exe`。部署时若把 target\\release 那个
    **名字**整个挪到根目录（`mv` 而非 `cp`），根目录与 target\\release 就成
    了同一个文件。此后 `shutil.copy2(src, dst)` 在 Windows 上抛
    WinError 32「另一个程序正在使用此文件」——与真实文件锁的报错**一字
    不差**，`except OSError` 分不出来，于是脚本会去结束用户正在运行的实例。

    修法是只换文件名、不碰任何进程：Windows 允许对运行中的 exe 改名
    （加载器以 FILE_SHARE_DELETE 打开），运行中的进程继续执行它自己那个
    inode，新名字指向的是刚写好的独立副本。任一步失败都回滚，绝不把根目录
    留成没有 exe 的状态。
    """
    marker = PROJ / (EXE_NAME + ".fresh")
    stash = PROJ / (EXE_NAME + ".hardlinked")
    try:
        shutil.copyfile(dst, marker)
    except OSError as e:
        print(f"无法复制根目录 exe（{e}）")
        return False

    try:
        os.replace(dst, stash)
    except OSError as e:
        marker.unlink(missing_ok=True)
        print(f"无法重命名根目录 exe（{e}）。请先关闭 cs2_font_changer 再重试。")
        return False

    try:
        os.replace(marker, dst)
    except OSError as e:
        os.replace(stash, dst)
        print(f"无法放置独立副本（{e}），已回滚")
        return False

    stash.unlink(missing_ok=True)
    print("已改为独立副本（未结束任何进程；运行中的实例继续跑它自己那份）")
    return True


def kill_running_instances(exe_path: Path) -> int:
    """结束正在运行的旧实例，返回结束的数量。

    为什么需要：编译产物要复制到项目根目录覆盖同名 exe，而正在运行的实例
    持有该文件的写锁，复制必然失败。本函数只在复制失败时兜底调用。

    安全性：只结束**可执行文件路径与目标完全一致**的进程（normcase 后逐字
    比较），不会误伤同名但跑在别处的程序。

    psutil 缺失时退回 PowerShell 的 CIM 查询（系统自带，无需装库）。
    """
    target = os.path.normcase(str(exe_path.resolve()))
    killed = 0

    try:
        import psutil
    except ImportError:
        psutil = None

    if psutil is not None:
        for proc in psutil.process_iter(["pid", "exe"]):
            try:
                exe = proc.info.get("exe")
                if exe and os.path.normcase(str(exe)) == target:
                    proc.kill()
                    proc.wait(timeout=5)
                    print(f"已结束旧实例：PID {proc.info['pid']}")
                    killed += 1
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.TimeoutExpired):
                continue
        return killed

    # 回退路径：PowerShell CIM 查询（wmic 在新版 Windows 已移除，不能用）
    query = (
        "Get-CimInstance Win32_Process -Filter \"Name='{}'\" | "
        "Where-Object {{ $_.ExecutablePath -and "
        "($_.ExecutablePath.ToLower() -eq '{}') }} | "
        "Select-Object -ExpandProperty ProcessId"
    ).format(exe_path.name, target.replace("'", "''"))
    try:
        out = subprocess.run(
            ["powershell", "-NoProfile", "-NonInteractive", "-Command", query],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        print(f"查询运行中实例失败（将退回复制重试）：{e}")
        return 0

    for line in out.stdout.splitlines():
        pid = line.strip()
        if pid.isdigit():
            try:
                subprocess.run(
                    ["taskkill", "/PID", pid, "/F"],
                    capture_output=True,
                    timeout=15,
                )
                print(f"已结束旧实例：PID {pid}")
                killed += 1
            except (OSError, subprocess.TimeoutExpired):
                continue
    return killed


def main() -> int:
    env = pin_env()
    cargo = find_cargo(env)
    if not cargo:
        print("未找到可用的 cargo，请先安装 Rust 工具链。")
        return 1

    result = subprocess.run([cargo, "build", "--release"], env=env)
    if result.returncode != 0:
        print("编译失败，请查看上方错误信息。")
        return 1

    src = PROJ / "target" / "release" / EXE_NAME
    dst = PROJ / EXE_NAME
    if not src.is_file():
        print(f"找不到编译产物：{src}")
        return 1

    if BUILD_ONLY:
        print(f"编译完成（--build-only）：{src}")
        print("跳过部署与清理。")
        return 0

    # 先排除「源即目标」的硬链接态：cargo uplift 会把 target\\release 下的
    # exe 硬链接到 deps\\ 里的产物；若之前用 mv 部署过，根目录与 target\\release
    # 就成了同一个文件，copy2 会报与真实文件锁一字不差的 WinError 32。
    # 单独识别它并把根目录改成独立副本（只换名字，不碰任何进程）。
    try:
        same = dst.exists() and os.path.samefile(src, dst)
    except OSError:
        same = False
    if same:
        print("根目录 exe 与编译产物是同一个文件（硬链接），改为独立副本...")
        if not break_deploy_hardlink(src, dst):
            return 1

    copied = False
    try:
        shutil.copy2(src, dst)
        copied = True
    except OSError:
        print("exe 正被运行中的实例占用，先结束旧实例...")
        n = kill_running_instances(dst)
        if n == 0:
            print("未找到占用该 exe 的实例（或无法结束），稍后重试复制")
        for _attempt in range(5):
            try:
                shutil.copy2(src, dst)
                copied = True
                break
            except OSError:
                # 进程退出到句柄真正释放之间有短暂窗口，留出时间
                time.sleep(2)
    if not copied:
        print("exe 文件被占用，无法更新。请先关闭正在运行的 cs2_font_changer 再重试。")
        return 1

    # 编译完成且 exe 已复制，开始清理杂物间
    print("编译完成，开始清理杂物间（只删未被当前构建引用的旧产物）...")
    target_root = PROJ / "target"
    if target_root.is_dir():
        clear_stale_deps(target_root, "release")
        clear_stale_deps(target_root, "debug")
    else:
        print("无需清理（target 不存在）")

    print(f"部署完成：{dst}")

    if LAUNCH:
        subprocess.Popen([str(dst)], cwd=str(PROJ))
    return 0


if __name__ == "__main__":
    sys.exit(main())

use std::env::var_os;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use color_eyre::eyre;
use color_eyre::eyre::{Context, Result};
use rust_i18n::t;
use walkdir::WalkDir;

use crate::command::CommandExt;
use crate::error::TopgradeError;
use crate::execution_context::ExecutionContext;
use crate::step::Step;
use crate::utils::require_option;
use crate::utils::which;
use crate::{config, output_changed_message};

fn get_execution_path() -> OsString {
    let mut path = OsString::from("/usr/bin:");
    path.push(var_os("PATH").unwrap());
    path
}

pub trait ArchPackageManager {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()>;
}

pub struct YayParu {
    executable: PathBuf,
    pacman: PathBuf,
}

impl ArchPackageManager for YayParu {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        if ctx.config().show_arch_news() {
            ctx.run_type()
                .execute(&self.executable)
                .arg("-Pw")
                .status_checked_with_codes(&[1, 0])?;
        }

        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("--pacman")
            .arg(&self.pacman)
            .arg("-Syu")
            .args(ctx.config().yay_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes(Step::System) {
            command.arg("--noconfirm");
        }
        command.status_checked()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("--pacman").arg(&self.pacman).arg("-Scc");
            if ctx.config().yes(Step::System) {
                command.arg("--noconfirm");
            }
            command.status_checked()?;
        }

        Ok(())
    }
}

impl YayParu {
    fn get(exec_name: &str, pacman: &Path) -> Option<Self> {
        Some(Self {
            executable: which(exec_name)?,
            pacman: pacman.to_owned(),
        })
    }
}

pub struct GarudaUpdate {
    executable: PathBuf,
}

impl ArchPackageManager for GarudaUpdate {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.executable);

        command
            .env("PATH", get_execution_path())
            .env("UPDATE_AUR", "1")
            .env("SKIP_MIRRORLIST", "1");

        if ctx.config().yes(Step::System) {
            command.env("PACMAN_NOCONFIRM", "1");
        }
        command.args(ctx.config().garuda_update_arguments().split_whitespace());
        command.status_checked()?;

        Ok(())
    }
}

impl GarudaUpdate {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("garuda-update")?,
        })
    }
}

pub struct Trizen {
    executable: PathBuf,
}

impl ArchPackageManager for Trizen {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("-Syu")
            .args(ctx.config().trizen_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes(Step::System) {
            command.arg("--noconfirm");
        }
        command.status_checked()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("-Sc");
            if ctx.config().yes(Step::System) {
                command.arg("--noconfirm");
            }
            command.status_checked()?;
        }

        Ok(())
    }
}

impl Trizen {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("trizen")?,
        })
    }
}

pub struct Pacman {
    executable: PathBuf,
}

impl ArchPackageManager for Pacman {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let sudo = require_option(ctx.sudo().as_ref(), "sudo is required to run pacman".into())?;
        let mut command = ctx.run_type().execute(sudo);
        command
            .arg(&self.executable)
            .arg("-Syu")
            .env("PATH", get_execution_path());
        if ctx.config().yes(Step::System) {
            command.arg("--noconfirm");
        }
        command.status_checked()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(sudo);
            command.arg(&self.executable).arg("-Scc");
            if ctx.config().yes(Step::System) {
                command.arg("--noconfirm");
            }
            command.status_checked()?;
        }

        Ok(())
    }
}

impl Pacman {
    pub fn get() -> Option<Self> {
        Some(Self {
            executable: which("powerpill").unwrap_or_else(|| PathBuf::from("pacman")),
        })
    }
}

pub struct Pikaur {
    executable: PathBuf,
}

impl Pikaur {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("pikaur")?,
        })
    }
}

impl ArchPackageManager for Pikaur {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("-Syu")
            .args(ctx.config().pikaur_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes(Step::System) {
            command.arg("--noconfirm");
        }

        command.status_checked()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("-Sc");
            if ctx.config().yes(Step::System) {
                command.arg("--noconfirm");
            }
            command.status_checked()?;
        }

        Ok(())
    }
}

pub struct Pamac {
    executable: PathBuf,
}

impl Pamac {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("pamac")?,
        })
    }
}
impl ArchPackageManager for Pamac {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("upgrade")
            .args(ctx.config().pamac_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes(Step::System) {
            command.arg("--no-confirm");
        }

        command.status_checked()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("clean");
            if ctx.config().yes(Step::System) {
                command.arg("--no-confirm");
            }
            command.status_checked()?;
        }

        Ok(())
    }
}

pub struct Aura {
    executable: PathBuf,
}

impl Aura {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("aura")?,
        })
    }
}

impl ArchPackageManager for Aura {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        use semver::Version;

        let version_cmd_output = ctx
            .run_type()
            .execute(&self.executable)
            .arg("--version")
            .output_checked_utf8()?;
        // Output will be something like: "aura x.x.x\n"
        let version_cmd_stdout = version_cmd_output.stdout;
        let version_str = version_cmd_stdout.trim_start_matches("aura ").trim_end();
        let version = Version::parse(version_str)
            .wrap_err_with(|| output_changed_message!("aura --version", "invalid version"))?;

        // Aura, since version 4.0.6, no longer needs sudo.
        //
        // https://github.com/fosskers/aura/releases/tag/v4.0.6
        let version_no_sudo = Version::new(4, 0, 6);

        if version >= version_no_sudo {
            let mut cmd = ctx.run_type().execute(&self.executable);
            cmd.arg("-Au")
                .args(ctx.config().aura_aur_arguments().split_whitespace());
            if ctx.config().yes(Step::System) {
                cmd.arg("--noconfirm");
            }
            cmd.status_checked()?;

            let mut cmd = ctx.run_type().execute(&self.executable);
            cmd.arg("-Syu")
                .args(ctx.config().aura_pacman_arguments().split_whitespace());
            if ctx.config().yes(Step::System) {
                cmd.arg("--noconfirm");
            }
            cmd.status_checked()?;
        } else {
            let sudo = crate::utils::require_option(
                ctx.sudo().as_ref(),
                t!("Aura(<0.4.6) requires sudo installed to work with AUR packages").to_string(),
            )?;

            let mut cmd = ctx.run_type().execute(sudo);
            cmd.arg(&self.executable)
                .arg("-Au")
                .args(ctx.config().aura_aur_arguments().split_whitespace());
            if ctx.config().yes(Step::System) {
                cmd.arg("--noconfirm");
            }
            cmd.status_checked()?;

            let mut cmd = ctx.run_type().execute(sudo);
            cmd.arg(&self.executable)
                .arg("-Syu")
                .args(ctx.config().aura_pacman_arguments().split_whitespace());
            if ctx.config().yes(Step::System) {
                cmd.arg("--noconfirm");
            }
            cmd.status_checked()?;
        }

        Ok(())
    }
}

fn box_package_manager<P: 'static + ArchPackageManager>(package_manager: P) -> Box<dyn ArchPackageManager> {
    Box::new(package_manager) as Box<dyn ArchPackageManager>
}

pub fn get_arch_package_manager(ctx: &ExecutionContext) -> Option<Box<dyn ArchPackageManager>> {
    let pacman = which("powerpill").unwrap_or_else(|| PathBuf::from("pacman"));

    match ctx.config().arch_package_manager() {
        config::ArchPackageManager::Autodetect => GarudaUpdate::get()
            .map(box_package_manager)
            .or_else(|| YayParu::get("paru", &pacman).map(box_package_manager))
            .or_else(|| YayParu::get("yay", &pacman).map(box_package_manager))
            .or_else(|| Trizen::get().map(box_package_manager))
            .or_else(|| Pikaur::get().map(box_package_manager))
            .or_else(|| Pamac::get().map(box_package_manager))
            .or_else(|| Pacman::get().map(box_package_manager))
            .or_else(|| Aura::get().map(box_package_manager)),
        config::ArchPackageManager::GarudaUpdate => GarudaUpdate::get().map(box_package_manager),
        config::ArchPackageManager::Trizen => Trizen::get().map(box_package_manager),
        config::ArchPackageManager::Paru => YayParu::get("paru", &pacman).map(box_package_manager),
        config::ArchPackageManager::Yay => YayParu::get("yay", &pacman).map(box_package_manager),
        config::ArchPackageManager::Pacman => Pacman::get().map(box_package_manager),
        config::ArchPackageManager::Pikaur => Pikaur::get().map(box_package_manager),
        config::ArchPackageManager::Pamac => Pamac::get().map(box_package_manager),
        config::ArchPackageManager::Aura => Aura::get().map(box_package_manager),
    }
}

pub fn upgrade_arch_linux(ctx: &ExecutionContext) -> Result<()> {
    let package_manager =
        get_arch_package_manager(ctx).ok_or_else(|| eyre::Report::from(TopgradeError::FailedGettingPackageManager))?;
    package_manager.upgrade(ctx)
}

pub fn show_pacnew() {
    let mut iter = WalkDir::new("/etc")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|f| {
            f.path()
                .extension()
                .filter(|ext| ext == &"pacnew" || ext == &"pacsave")
                .is_some()
        })
        .peekable();

    if iter.peek().is_some() {
        println!("\n{}", t!("Pacman backup configuration files found:"));

        for entry in iter {
            println!("{}", entry.path().display());
        }
    }
}

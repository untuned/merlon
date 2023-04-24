use clap::Parser;
use anyhow::Result;

mod new;
mod pack;
mod apply;

/// Mod manager for the Paper Mario (N64) decompilation.
/// 
/// Merlon allows you to create mods that can be applied to the decomp source code, and to package mods
/// into `.merlon` files that can be applied to a copy of the decomp source code.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Create a new mod.
    ///
    /// This will create a new git repository in the specified directory.
    /// The repository will have a `papermario` submodule, which will be set to the latest commit on the `main` branch.
    New(new::Args),

    /// Package a mod for distribution.
    ///
    /// Mods are distributed as `.merlon` files, which are encrypted, compressed tarballs of git patches.
    /// Merlon files are encrypted using the base ROM as the key.
    /// The patches are applied to the `papermario` submodule in the mod's directory.
    Pack(pack::Args),

    /// Apply a mod package.
    Apply(apply::Args),
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcmd {
        SubCommand::New(new_args) => new::run(new_args),
        SubCommand::Pack(package_args) => pack::run(package_args),
        SubCommand::Apply(apply_args) => apply::run(apply_args),
    }
}

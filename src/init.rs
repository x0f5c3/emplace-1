use crate::{config::Config, public_clap_app};
use anyhow::{anyhow, Context, Result};
use clap_complete::shells;
use std::{env, io, path::Path};

pub fn init_main<P>(config_path: P, shell_name: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let exe_path = env::current_exe()
        .context("getting executable path for initial configuration")?
        .into_os_string()
        .into_string()
        .expect("Could not convert path to string");

    let setup_script = match shell_name {
        "bash" => BASH_INIT,
        "zsh" => ZSH_INIT,
        "fish" => FISH_INIT,
        "nu" => NU_INIT,
        _ => panic!("Shell \"{}\" is not supported at the moment", shell_name),
    };

    println!(
        "{}",
        // Replace command
        setup_script
            .replace("## EMPLACE ##", exe_path.as_str())
            // Replace path
            .replace(
                "## EMPLACE_CONFIG_PATH ##",
                config_path
                    .as_ref()
                    .canonicalize()
                    // When canonicalizing path failed use the default path
                    .unwrap_or_else(|_| Config::default_path())
                    .to_str()
                    .ok_or_else(|| {
                        anyhow!(
                            "Could not convert path {:?} to string",
                            config_path.as_ref()
                        )
                    })?,
            )
    );

    // Print the completions
    match shell_name {
        "bash" => clap_complete::generate(
            shells::Bash,
            &mut public_clap_app(),
            "emplace",
            &mut io::stdout(),
        ),
        "zsh" => clap_complete::generate(
            shells::Zsh,
            &mut public_clap_app(),
            "emplace",
            &mut io::stdout(),
        ),
        "fish" => clap_complete::generate(
            shells::Fish,
            &mut public_clap_app(),
            "emplace",
            &mut io::stdout(),
        ),
        _ => (),
    };

    Ok(())
}

const BASH_INIT: &str = r###"
emplace_postexec_invoke_exec () {
    # quit when the previous command failed
    [ -z "$?" ] && exit $?

    local hist=`history 1`

    local this_command=`HISTTIMEFORMAT= echo $hist | sed -e "s/^[ ]*[0-9]*[ ]*//"`;
    ## EMPLACE ## catch "$this_command"
}
PROMPT_COMMAND="emplace_postexec_invoke_exec;$PROMPT_COMMAND"

export EMPLACE_CONFIG="## EMPLACE_CONFIG_PATH ##"

# Add an empty line so new shells won't display the last installable command
echo "# emplace" >> $HISTFILE
"###;

const ZSH_INIT: &str = r###"
emplace_precmd() {
    # quit when the previous command failed
    [ -z "$?" ] && exit

    local hist=`history -1`

    local this_command=`HISTTIMEFORMAT= echo $hist | sed -e "s/^[ ]*[0-9]*[ ]*//"`;
    ## EMPLACE ## catch "$this_command"
}
# Don't hook them double in nested shells
if [[ ${precmd_functions[(ie)emplace_precmd]} -gt ${#precmd_functions} ]]; then
    precmd_functions+=(emplace_precmd)
fi

export EMPLACE_CONFIG="## EMPLACE_CONFIG_PATH ##"

# Add an empty line so new shells won't display the last installable command
echo "# emplace" >> $HISTFILE
"###;

const FISH_INIT: &str = r###"
function emplace_postcmd --on-event fish_postexec
    # quit when the previous command failed
    if test $status -gt 0
        return
    end

    ## EMPLACE ## catch "$argv"
end

set -x EMPLACE_CONFIG "## EMPLACE_CONFIG_PATH ##"
"###;

const NU_INIT: &str = r###"
EMPLACE_CONFIG="## EMPLACE_CONFIG_PATH ##" ## EMPLACE ## catch $(history | last); echo >
"###;

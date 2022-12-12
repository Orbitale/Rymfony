use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::process::ExitCode;

pub(crate) struct CommandList {
    pub(crate) commands: Vec<Box<CommandHandler>>,
}

impl CommandList {
    pub(crate) fn subcommands(&self) -> Vec<ClapCommand> {
        self.commands.iter().fold(Vec::new(), |mut commands, command| {
            commands.push(command.command_definition.clone());
            commands
        })
    }
}

pub(crate) struct CommandHandler {
    pub(crate) command_definition: ClapCommand,
    pub(crate) executor: Box<dyn Fn(&ArgMatches) -> ExitCode>,
}

impl CommandHandler {
    pub(crate) fn new(command_definition: ClapCommand, executor: Box<dyn Fn(&ArgMatches) -> ExitCode>) -> Self {
        Self { command_definition, executor }
    }
}

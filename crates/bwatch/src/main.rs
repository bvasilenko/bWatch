mod invocation;
mod runtime;

use bsuite_core::{
    BsuiteCoreError, EmitFormat, ExitCode, ProcessExitEmitter, prompt_resolver::DirectiveString,
};
use bwatch::{BwatchCli, BwatchError, Cmd, FindingCategory};
use clap::{Parser, error::ErrorKind};
use invocation::InvocationTranscript;
use runtime::BinaryRuntime;
use std::io::Write as _;
use std::path::PathBuf;

fn main() {
    let cli = match BwatchCli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = clap_exit_code(&error);
            let _ = error.print();
            std::process::exit(exit_code.as_i32());
        }
    };
    let format = emit_format_for(&cli.command);
    let mut emitter = ProcessExitEmitter::new(format);

    let exit_code = match init_and_run(cli) {
        Ok(CommandOutcome::Directive {
            directive,
            exit_code,
        }) => emitter.emit_directive(Ok((directive, exit_code))),
        Ok(CommandOutcome::Silent(exit_code)) => exit_code,
        Err(RunError::Malformed(e)) => {
            let _ = writeln!(std::io::stderr(), "{e}");
            ExitCode::Usage
        }
        Err(RunError::Internal(e)) => emitter.emit_directive(Err(e)),
    };

    std::process::exit(exit_code.as_i32());
}

fn clap_exit_code(error: &clap::Error) -> ExitCode {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => ExitCode::Success,
        _ => ExitCode::Usage,
    }
}

fn init_and_run(cli: BwatchCli) -> Result<CommandOutcome, RunError> {
    let runtime = BinaryRuntime::init(install_dir()).map_err(RunError::Internal)?;
    let invocation = InvocationTranscript::start(
        runtime.host_context,
        runtime.invocation_context.clone(),
        runtime.corpus_version,
    );
    run(cli, runtime, invocation)
}

fn run(
    cli: BwatchCli,
    runtime: BinaryRuntime,
    invocation: InvocationTranscript,
) -> Result<CommandOutcome, RunError> {
    match cli.command {
        Cmd::Poll(args) => {
            let result = bwatch::poll::run(&args, &runtime.corpus).map_err(classify_bwatch_error);
            let exit_code = result
                .as_ref()
                .map_or_else(|e| e.exit_code(), |(_, code)| *code);
            invocation.flush(&runtime.appender, exit_code, result.is_ok());
            result.map(|(directive, exit_code)| CommandOutcome::Directive {
                directive,
                exit_code,
            })
        }
        Cmd::FindingCategories => {
            let listing = FindingCategory::ALL
                .iter()
                .map(|cat| cat.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            invocation.flush(&runtime.appender, ExitCode::Success, false);
            Ok(CommandOutcome::Directive {
                directive: DirectiveString::new(listing),
                exit_code: ExitCode::Success,
            })
        }
        Cmd::Update => {
            let result = bwatch::update::run(&runtime.install_dir)
                .map_err(|e| RunError::Internal(e.into_core()));
            let exit_code = result
                .as_ref()
                .map_or_else(|e| e.exit_code(), |()| ExitCode::Success);
            invocation.flush(&runtime.appender, exit_code, false);
            result.map(|()| CommandOutcome::Silent(ExitCode::Success))
        }
        Cmd::Init | Cmd::Tail | Cmd::Explain | Cmd::Process => {
            invocation.flush(&runtime.appender, ExitCode::Success, false);
            Ok(CommandOutcome::Silent(ExitCode::Success))
        }
    }
}

fn emit_format_for(command: &Cmd) -> EmitFormat {
    match command {
        Cmd::Poll(args) if args.json => EmitFormat::Json,
        _ => EmitFormat::Plain,
    }
}

fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn classify_bwatch_error(error: BwatchError) -> RunError {
    if error.is_malformed_input() {
        RunError::Malformed(error)
    } else {
        RunError::Internal(error.into_core())
    }
}

#[derive(Debug)]
enum CommandOutcome {
    Directive {
        directive: DirectiveString,
        exit_code: ExitCode,
    },
    Silent(ExitCode),
}

#[derive(Debug)]
enum RunError {
    Malformed(BwatchError),
    Internal(BsuiteCoreError),
}

impl RunError {
    fn exit_code(&self) -> ExitCode {
        match self {
            Self::Malformed(_) => ExitCode::Usage,
            Self::Internal(_) => ExitCode::InternalError,
        }
    }
}

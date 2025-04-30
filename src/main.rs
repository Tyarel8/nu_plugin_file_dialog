use std::path::PathBuf;

use native_dialog::DialogBuilder;
use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(FileDialogCommand)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}

struct FileDialogCommand;

impl SimplePluginCommand for FileDialogCommand {
    type Plugin = FileDialogPlugin;

    fn name(&self) -> &str {
        "file-dialog"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_types(vec![
                (Type::Nothing, Type::String),
                (Type::Nothing, Type::List(Box::new(Type::String))),
            ])
            .switch("multiple", "Select multiple values", Some('m'))
            .switch("dir-only", "Select a directory instead of files", Some('d'))
            .named(
                "base-dir",
                SyntaxShape::Directory,
                "Base dir to search",
                Some('b'),
            )
            .named("title", SyntaxShape::String, "Window title", Some('t'))
            .named(
                "filter",
                SyntaxShape::Record(vec![]),
                "Filters to use",
                Some('f'),
            )
    }

    fn examples(&self) -> Vec<nu_protocol::Example> {
        vec![Example {
            example: "file-dialog -m -b ~/Images -f {Normal: [png, jpg], Weird: [webp]}",
            description: "Select multiple images in the ~/Images folder",
            result: None,
        }]
    }

    fn description(&self) -> &str {
        "Select file(s) using the native dialog"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_protocol::LabeledError> {
        let select_dir = call.has_flag("dir-only")?;
        let base_dir = match call.get_flag_value("base-dir") {
            Some(Value::String { val, .. }) if PathBuf::from(&val).is_dir() => Some(val),
            None => Some(engine.get_current_dir()?),
            _ => {
                return Err(LabeledError::new("dir has an incorrect type/path")
                    .with_label("dir has to be a directory", call.head))
            }
        };
        let title = match call.get_flag_value("title") {
            Some(Value::String { val, .. }) => Some(val),
            None => None,
            _ => {
                return Err(LabeledError::new("title has an incorrect type")
                    .with_label("title has to be a string", call.head))
            }
        };

        let record_error = Err(LabeledError::new("filter has an incorrect type")
            .with_label("filter has to be a record of List(String)", call.head));

        let multiple = call.has_flag("multiple")?;
        let filter = match call.get_flag_value("filter") {
            Some(Value::Record { val, .. }) => Some(val),
            None => None,
            _ => return record_error,
        };

        let mut fd = DialogBuilder::file();

        // Add filters
        let filter_values: Option<Vec<(String, Vec<&str>)>> = if let Some(f) = &filter {
            let mut fv = vec![];

            for (key, value) in f.iter() {
                match value {
                    Value::List { vals, .. } => {
                        let mut filters = vec![];

                        for v in vals {
                            match v {
                                Value::String { val, .. } => filters.push(val.as_str()),
                                _ => return record_error,
                            }
                        }

                        fv.push((key.to_string(), filters));
                    }
                    _ => return record_error,
                }
            }

            Some(fv)
        } else {
            None
        };

        if let Some(fv) = &filter_values {
            for (key, value) in fv {
                fd = fd.add_filter(key, value);
            }
        }

        // Add title
        if let Some(t) = &title {
            fd = fd.set_title(t);
        }

        // Add base dir
        if let Some(bd) = &base_dir {
            fd = fd.set_location(bd);
        }

        // Show dialog and get result
        let result = match (select_dir, multiple) {
            (true, true) => {
                return Err(
                    LabeledError::new("Cannot select multiple directories").with_label(
                        "Only one of `--multiple` or `--only-dir` can be used",
                        call.head,
                    ),
                )
            }
            (false, true) => {
                return Ok(Value::list(
                    fd.open_multiple_file()
                        .show()
                        .expect("Can't show dialog")
                        .iter()
                        .map(|p| {
                            Value::string(
                                p.clone()
                                    .into_os_string()
                                    .into_string()
                                    .expect("Non utf-8 path"),
                                call.head,
                            )
                        })
                        .collect::<Vec<Value>>(),
                    call.head,
                ))
            }
            (true, false) => fd.open_single_dir().show().expect("Can't show dialog"),
            (false, false) => fd.open_single_file().show().expect("Can't show dialog"),
        };

        Ok(result
            .map(|p| {
                Value::string(
                    p.clone()
                        .into_os_string()
                        .into_string()
                        .expect("Non utf-8 path"),
                    call.head,
                )
            })
            .unwrap_or_else(|| Value::string("", call.head)))
    }
}

fn main() {
    serve_plugin(&FileDialogPlugin, MsgPackSerializer)
}

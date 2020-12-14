use structopt::StructOpt;

use crate::input::InputConfig;
use crate::output::OutputArgs;
use crate::report::ReportServiceConfig;

#[derive(Clone, Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub input_config: InputConfig,
    #[structopt(flatten)]
    pub output_config: OutputArgs,
    #[structopt(flatten)]
    pub report_config: ReportServiceConfig,
}

use clap::{Parser,Subcommand};
use std::io::ErrorKind;
use std::fs;


#[derive(Parser,Debug)]
#[command(name = "cli_ai",version,about)]


pub struct Cli{
    #[command(subcommand)]
    command:Commands,
}


#[derive(Subcommand)]

pub enum Commands{
    Run{

    cli_ai :String,

    #[arg(last = true)]
    args: Vec<String>,

    #[arg(long,default_value_t = 5)]
    context:usize,

    #[arg(long)]
    mock:bool,

    
    
    #[arg(long,default_value = "cli_ai_bugreports")]
    report_dir:String,

    #[arg(long,default_value_t = 5)]
    context:usize,

    #[arg(long)]
    no_reports:bool,
    
    #[arg(long, default_value_t = 15)]
    timeout: u64,
    
    
    }
    
}

fn main(){
    let cli = Cli::parse();
    match &cli.command{

        Commands::Run{cli_ai} => {

            match fs::read_to_string(cli_ai) {

                Ok(contents) => println!("File contents :\n{}",contents),
                
                Err(error) => match error.kind(){
                    ErrorKind::NotFound => {
                        eprintln!("error:the file '{}' was not found",cli_ai);
                    }
                    ErrorKind::PermissionDenied =>{
                        eprintln!("you dont have permission fo acessing file '{}' ",cli_ai);

                    }
                    _ =>{
                        eprintln!("an unexpected error occured while reading '{}' :{}",cli_ai,error);
                    }
                }
                
                
            }
        }
    }
}
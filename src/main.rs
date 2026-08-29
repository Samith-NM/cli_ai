use color_eyre::eyre::{Ok, Result};
use ratatui::{DefaultTerminal, crossterm::event::{self, Event::{self, Key}}, widgets::Paragraph};
fn main() -> Result<()>{

    println!("Hello, world!");
    color_eyre::install()?;
    
    let terminal = ratatui::init();
    
    let result= run(terminal);
    ratatui::restore();
    result
    

}
fn run(mut terminal: DefaultTerminal)->Result<()>{
    loop{
        terminal.draw(render)

        if let Event::Key(key) = event::read()?   {
            match key.code {
                event::KeyCode::Esc =>{
                    break;
                }
                event::KeyCode::Up => todo!(),
                event::KeyCode::Down => todo!(),
                event::KeyCode::Left => todo!(),
                event::KeyCode::Right => todo!(),
                event::KeyCode::Backspace => todo!(),
                
                
                
                event::KeyCode::Tab => todo!(),
                event::KeyCode::Delete => todo!(),
                _ =>{ },
                
              
               
            }
            
        }
    }

    Ok(())
}
fn render(frame:&mut Frame){
    Paragraph::new("hello from application ").render(frame.area(),frame.buffer_mut());

}
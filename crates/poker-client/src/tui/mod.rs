use std::{error::Error, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    prelude::{Backend, CrosstermBackend},
    widgets::{Paragraph, Wrap},
};

use crate::{
    state::{AppState, state_worker::check_worker_logic},
    worker::WorkerRequest,
};

mod inputhandler;
mod keyhandler;
mod layoututils;
mod screens;
mod tuidraw;

/// This will setup the terminal so we can start our TUI app
/// Then call the main loop to actually run the app
/// After the main loop, terminal teardown will take place
/// All this may throw an error, which should be checked in the main fn
pub fn start_tui(state: AppState) -> Result<(), Box<dyn Error>> {
    // Setup terminal, so we can have our TUI screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Now we call our main loop
    let finished = run_pokerkees(&mut terminal, state);

    // restore terminal stuff
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Log propagated errors, if appropriate
    if finished.is_ok() {
        log::info!("Tui closed");
    } else if let Err(err) = finished {
        log::error!("Exited with error.");
        log::error!("{err}");
    }

    Ok(())
}

fn run_pokerkees<B: Backend>(terminal: &mut Terminal<B>, mut state: AppState) -> io::Result<bool>
where
    io::Error: From<B::Error>,
{
    // this function contains our app loop
    // it should not contain logic really
    // just handle calls

    // Some setup
    let timeout = Duration::from_millis(100);
    // if we have a server url, we would like to perform
    // a connection check right away
    if state.server_url.is_some() {
        state
            .worker
            .tasks
            .push_back(WorkerRequest::CheckServerConnection(
                state.server_url.clone(),
            ));
    }

    loop {
        // Exit state
        if state.should_exit {
            return Ok(true);
        }

        // Always start with a draw
        // If draw fails, we likely have serious error
        // So propagate, leading to exit of app
        draw_pokerkees(terminal, &mut state)?;

        // Listen for user input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // handle_key_press
                handle_key_event(key, &mut state);
            }
        }

        // Check worker logic
        check_worker_logic(&mut state);
    }
}

fn draw_pokerkees<B: Backend>(terminal: &mut Terminal<B>, state: &mut AppState) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
    // TODO, handle draw errors
    terminal.draw(|frame| {
        if frame.area().width < 100 || frame.area().height < 30 {
            frame.render_widget(
                Paragraph::new("The screen is too small, increase size or decrease resolution")
                    .wrap(Wrap { trim: false }),
                frame.area(),
            );
            return;
        }
        // Offload the real drawing to dedicated module
        tuidraw::draw_ui(frame, state);
    })?;
    Ok(())
}

fn handle_key_event(key: KeyEvent, state: &mut AppState) {
    // For readability reasons we move the call to another mod

    // Do some simple checks
    // Ignore if release to prevent double events
    // on some systems
    if key.is_release() {
        return;
    }
    keyhandler::handle_key(key, state);
}

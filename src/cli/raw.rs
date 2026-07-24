use std::io::Write;
use std::str::FromStr;

use crate::cli::HandleEvent;

#[derive(Default)]
pub struct RawCli {
    waiter: crate::utils::Waiter,
    input: crate::cli::Input,
    client: crate::network::Client,
    player: crate::game::Player,
    command: Option<crate::messages::Command>,
}

impl RawCli {
    pub async fn start(&mut self, client: crate::network::Client)  -> Option<crate::messages::Error> {
        self.waiter.begin(3);
        self.client = client;
        crate::cli::Handler::init();
        let r = self.run().await;
        crate::cli::Handler::cleanup();
        r
    }

    async fn run(&mut self) -> Option<crate::messages::Error> {
        fn clear_line() {
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveToColumn(0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            );
        }

        fn print_input(cli: &RawCli) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::style::Print(format!(
                    "[{} proto={}] {}: ",
                    cli.client.addr,
                    cli.client.proto,
                    if cli.player.username.is_empty() { "?" } else { &cli.player.username },
                )),
                crossterm::style::Print(&cli.input.buffer),
            );
            let _ = std::io::stdout().flush();
        }

        fn print_out(cli: &RawCli, from: char, s: &str) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::style::Print(format!("{from}: ")),
                crossterm::style::Print(s),
                crossterm::style::Print("\n"),
            );
            if cli.client.is_open() {
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::style::Print("\n"),
                );
                print_input(cli);
            }
            let _ = std::io::stdout().flush();
        }

        fn print_err(cli: &RawCli, error: &crate::messages::Error) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stderr(),
                crossterm::style::Print("S: "),
                crossterm::style::Print(error.to_string()),
                crossterm::style::Print("\n\n"),
            );
            print_input(cli);
            let _ = std::io::stderr().flush();
        }

        print_input(self);
        loop {
            if !self.client.is_open() {
                return None;
            }
            if let Some(e) = tokio::select! {
                _ = self.waiter.wait() => Some(crate::messages::Error::ServerTimeOut),
                event = self.input.handle_event() => {
                    if let Some(event) = event {
                        match event {
                            crate::cli::Event::Interrupted => return None,
                            crate::cli::Event::Validate if !self.waiter.is_waiting() => {
                                let input = self.input.consume();
                                print_out(self, 'C', &input);
                                self.waiter.begin(3);
                                if let Ok(command) = crate::messages::Command::from_str(&input) {
                                    self.command = Some(command);
                                }
                                if let Some(writer) = &self.client.writer {
                                    match writer.write(&format!("{}\n", input)).await {
                                        Ok(_) => None,
                                        Err(_) => Some(crate::messages::Error::SendFailed),
                                    }
                                } else {
                                    Some(crate::messages::Error::SendFailed)
                                }
                            }
                            _ => {
                                print_input(self);
                                None
                            }
                        }
                    } else {
                        print_input(self);
                        None
                    }
                }
                message = self.client.reader.read() => {
                    match message {
                        Ok(Some(message)) => match crate::messages::Message::from_str(&message) {
                            Ok(message) => {
                                self.waiter.end();
                                match message {
                                    crate::messages::Message::Error(error) => Some(error),
                                    crate::messages::Message::Response(response) => {
                                        let r = self.process_response(&response);
                                        print_out(self, 'S', &response.to_string());
                                        r
                                    }
                                    crate::messages::Message::Event(event) => {
                                        print_out(self, 'S', &event.to_string());
                                        None
                                    }
                                    crate::messages::Message::Command(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                                }
                            }
                            Err(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                        }
                        _ => Some(crate::messages::Error::ConnectionClosed),
                    }
                }
            } {
                if e.is_fatal() {
                    clear_line();
                    return Some(e);
                } else {
                    print_err(self, &e);
                }
            }
        }
    }

    fn process_response(&mut self, response: &crate::messages::Response) -> Option<crate::messages::Error> {
        match &self.command {
            Some(command) => match command.kind {
                crate::messages::CommandKind::Connect => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                } else {
                    self.client.state = crate::network::ClientState::Authenticated;
                    self.player.username.clear();
                    if command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut self.player.username),
                    ]).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                }
                crate::messages::CommandKind::Quit => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "bye".to_string()),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                } else {
                    self.client.close();
                }
                _ => (),
            }
            None => {
                self.client.proto.clear();
                if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "hello".to_string()),
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
            }
        };
        self.command = None;
        None
    }
}

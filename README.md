*This project has been created as part of the 42 curriculum by mosmond, tbaricau*

# TAP

## Description

The goal of this project is to create a MUD (Muti-User Dungeon) whether the communication between server and client use TAP (The-Answer-Protocol).

## Features

- Custom worlds
- Raw commands client
- TUI client
- GUI client
- Quests
- Combat
- Chat

## Instructions

You must install Rust and Cargo before trying to build this project.
<https://doc.rust-lang.org/cargo/getting-started/installation.html>


Compile the project
```Shell
cargo build --release
```

Run the server
```Shell
cargo run-server
```

Run the server with custom parameters
```Shell
cargo run --bin server -- <your_world_file> [--ip <bind_ip>] [--port <bind_port>]
```

Run the raw client
```Shell
cargo run-client-raw
```

Run the TUI client
```Shell
cargo run-client-tui
```

Run the GUI client
```Shell
cargo run-client-gui
```

Run the client with custom parameters
```Shell
cargo run --bin server -- <your_world_file> [--raw] [--gui] [--ip <server_ip>] [--port <server_port>]
```

Check code quality
```Shell
cargo lint
```

Clean project directory
```Shell
cargo clean
```

## Architecture

The server's main thread handles client connections. When a new connection occurs, an asynchronous function is launched to manage the client. This function contains a loop in which the server waits for a command and then passes it to the appropriate logic based on the command name.

There is a struct that represent the game state. This struct is protected by a mutex to avoid conflicting modifications.

## Protocol Implementation

As the protocol is a bit limited, we added several command, errors and events.

Commands we added:
- ``CONSUME``: to consume a healing item, as the protocol does not allow players to heal themselves
- ``DESCRIBE``: to retrieve the JSON data associated to a world element id, as the protocol just returns ids of things in commands like ``INVENTORY`` or ``LOOK``
- ``EQUIP``: to allow players to equip equipable items, to create better combat system
- ``GROUP DESCRIBE``: to retrive informations about the current player group, as the protocol doesn't give any command to do something similar
- ``UNEQUIP``: mirror command of ``EQUIP``

Errors we added:
- ``UNKNOWN_ERROR (1)``: for unknown errors
- ``NOT_A_COMMAND (100)``: server receive something that cannot by parsed as a command
- ``UNKNOWN_COMMAND (101)``: command name doesn't exists
- ``INVALID_ARGUMENTS (102)``: arguments given for a command are invalid
- ``ALREADY_AUTHENTICATED (103)``: try to connect but already authenticated
- ``NOT_AUTHENTICATED (104)``: try to execute a command when not authenticated
- ``INVALID_NAME (202)``: username used is invalid
- ``NOT_INVITED (203)``: trying to join a group when not invited to
- ``PLAYER_NOT_FOUND (404)``: requested player not found
- ``GROUP_NOT_FOUND (404)``: requested group not found
- ``ITEM_NOT_FOUND (404)``: requested item not found
- ``QUEST_NOT_FOUND (404)``: requested quest not found
- ``NPC_NOT_NEUTRAL (407)``: trying to talk or ask quest to non neutral enemy
- ``ITEM_NOT_CONSUMABLE (408)``: item cannot be consume
- ``ITEM_NOT_EQUIPABLE (409)``: item cannot be equiped
- ``QUEST_NOT_ACTIVE (410)``: trying to abandon not active quest
- ``IN_COMBAT (411)``: player or target is in combat
- ``CONNECTION_CLOSED (902)``: connection between server and client closed
- ``UNEXPECTED_SERVER_RESPONSE (903)``: server response doesn't match expected
- ``SERVER_TIMEOUT (904)``: server take too long to respond
- ``SERVER_ERROR (905)``: server error

Events we added:
- ``PLAYER DIE``: player died in combat
- ``PLAYER QUEST COMPLETE``: player quest was completed
- ``ROOM COMBAT END``: all enemies has been defeated
- ``ROOM COMBAT STATS``: combat status change

In the RFC, the "players stats" event format was the following: ``EVT STATS players=``. We just ajust it to be more coherent we others events by adding to event scope: ``EVT STATS PLAYERS players=``.

## Combat System

Each player or enemy has an armor score and an attack score.

When an attack is made, the damage inflicted is equal to the attacker's attack value minus the defender's armor value.

In a combat, each player can attack the enemy he wants.

When an enemy is attacked, it immediately retaliates.

## Quest System

Some NPCs can assign quests to players.

There are four types of quests:
- Quest that requires to bring a certain amount of item to the quest giver
- Quest that requires to go to a specific room
- Quest that requires to kill a certain amount of enemies
- Quest that requires to talk to a specific NPC

Once the quest objective is completed, players must return to speak with the NPC who gave the quest, unless it completes automatically. Next, the reward items are added to the player's inventory.

## World Design

The current map is directly inspired by the manga Shangri-La Frontier but any other detail is an original creation.
The quests are around the "old woman" last quest with the "old man" rewards wich are linked with the 7 sins and the idea of a better past.
The cities contains NCPs who give quests or dialogue and valuable items and between each city lies a wild zone with unique items and enemies to defeat.
The enemies represent the future of the world: sinless, strong and young.  

## Server Logging

All server logs are in format:

``[timestamp] TYPE: Message``

``INFO`` and ``WARN`` logs are output to stdout and ``ERROR`` logs are output to stderr.

## Group Contributions

Roles:
- mosmond
    - GUI Client
    - World building
- tbaricau
    - Protocol implementation
    - Game logic
    - CLI

## Building and Running

We simply use ``cargo`` which is the Rust integrated project manager.

To run each component, refer to the "Instructions" section or go into ``target`` directory and run executables directly.

## Testing

To test multipler functionality, run the server and open two more terminals, in each terminal run the client and then test whatever you want.

You can also create a custom simple world and run the server with:
```Shell
cargo run --bin server -- <your_world_file>
```

## Resources

Cargo installation:

<https://doc.rust-lang.org/cargo/getting-started/installation.html>

Rust documentation:

<https://doc.rust-lang.org/stable/std/index.html>

AI was used to understand how Rust work and to get examples of codes.

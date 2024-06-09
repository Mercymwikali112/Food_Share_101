# Decentralized Surplus Food Management System

This project is a decentralized platform built on the Internet Computer for managing surplus food donations. It allows donors to post surplus food, receivers to claim food, and drivers to manage deliveries. The platform ensures robust access control and user management.

## Key Features

1. **Donor Management**
   - **Add Donor:** Allows users to create donor profiles.
   - **Get All Donors:** Retrieve a list of all donor profiles.

2. **Receiver Management**
   - **Add Receiver:** Allows users to create receiver profiles.
   - **Get All Receivers:** Retrieve a list of all receiver profiles.

3. **Driver Management**
   - **Add Driver:** Allows users to create driver profiles.
   - **Get All Drivers:** Retrieve a list of all driver profiles.

4. **Surplus Post Management**
   - **Add Surplus Post:** Allows donors to post details of surplus food.
   - **Get All Surplus Posts:** Retrieve a list of all surplus food posts.
   - **Get Surplus Post by Food Type:** Retrieve surplus food posts filtered by food type.

5. **Assignment Management**
   - **Create Assignment:** Assigns a surplus post to a receiver and a driver.
   - **Get All Assignments:** Retrieve a list of all assignments.

6. **Surplus Record Management**
   - **Create Surplus Record:** Records the delivery of a surplus post by a driver.

7. **Error Handling**
   - **Not Found:** Returns an error if a requested item is not found.
   - **Unauthorized Access:** Returns an error if a user tries to perform an action without necessary permissions.

## Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:
```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```
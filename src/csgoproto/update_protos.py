import subprocess

repo_url = "https://github.com/SteamDatabase/Protobufs/"


try:
    # Run the git clone command
    subprocess.run(["git", "clone", repo_url], check=True)
    print(f"Successfully cloned {repo_url}")
except subprocess.CalledProcessError as e:
    print(f"An error occurred while cloning the repository: {e}")
except FileNotFoundError:
    print("Git is not installed or not found in the system's PATH.")

try:
    # Run the cargo run --release command
    result = subprocess.run(["cargo", "run", "--release"], check=True)
    print("Cargo run completed successfully.")
except subprocess.CalledProcessError as e:
    print(f"An error occurred while running the Cargo command: {e}")
except FileNotFoundError:
    print("Cargo is not installed or not found in the system's PATH.")


imports = """
pub mod cstrike15_gcmessages;
pub mod cstrike15_usermessages;
pub mod demo;
pub mod cs_usercmd;
pub mod engine_gcmessages;
pub mod gcsdk_gcmessages;
pub mod netmessages;
pub mod network_connection;
pub mod networkbasetypes;
pub mod steammessages;
pub mod usermessages;
pub mod usercmd;
"""

with open("src/lib.rs", "w") as f:
    f.write(imports)

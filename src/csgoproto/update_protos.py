import subprocess

repo_url = "https://github.com/SteamDatabase/GameTracking-CS2/"


try:
    # Run the git clone command
    subprocess.run(["git", "clone", repo_url, "--depth=1"], check=True)
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

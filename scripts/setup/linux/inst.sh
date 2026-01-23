# Check if required tools are installed
echo "Checking for required tools..."
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi
if ! command -v git &> /dev/null; then
    echo "Error: git is not installed. Please install git first."
    exit 1
fi

# Set installation directory to current directory
echo "Installation directory set to current directory: $(pwd)"
install_dir="$(pwd)"

# Clone or update repos
echo "Cloning or updating repositories..."
mkdir -p "$install_dir"
cd "$install_dir"

# Function to clone or pull repository
clone_or_pull() {
    local repo_url="$1"
    local repo_name=$(basename "$repo_url")

    if [ -d "$repo_name" ]; then
        echo "Repository $repo_name already exists, pulling latest changes..."
        cd "$repo_name"
        git pull origin main
        cd ..
    else
        echo "Cloning $repo_name..."
        git clone "$repo_url"
    fi
}

# Clone or update repositories
clone_or_pull https://github.com/JustEnoughVCS/CommandLine
clone_or_pull https://github.com/JustEnoughVCS/VersionControl

# Setup VersionControl repo
echo "Setting up VersionControl..."
cd VersionControl
chmod +x setup.sh
./setup.sh

# Build CLI
echo "Building CLI..."
cd ../CommandLine
chmod +x deploy.sh
./scripts/dev/deploy.sh

# Configure shell to include CLI in PATH
echo "Now adding JustEnoughVCS CLI to your environment. Please select your target shell:"
echo "1) ~/.zshrc (Zsh)"
echo "2) ~/.bashrc (Bash)"
echo "3) ~/.config/fish/config.fish (Fish)"
echo "4) Skip shell configuration"
echo -n "Enter your choice (1-4): "
read choice

case $choice in
    1)
        config_file="$HOME/.zshrc"
        ;;
    2)
        config_file="$HOME/.bashrc"
        ;;
    3)
        config_file="$HOME/.config/fish/config.fish"
        ;;
    4)
        echo "Skipping shell configuration."
        echo "Installation completed! You can manually add the CLI to your PATH later."
        exit 0
        ;;
    *)
        echo "Invalid choice. Skipping shell configuration."
        exit 0
        ;;
esac

cli_path="$(pwd)/export/jv_cli.sh"
echo "# JustEnoughVCS CLI" >> "$config_file"
echo "source \"$cli_path\"" >> "$config_file"
echo "CLI has been added to $config_file"
echo "Please restart your shell or run: source $config_file"
echo "Installation completed successfully!"

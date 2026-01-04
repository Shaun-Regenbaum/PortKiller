#!/bin/bash
# Generate process icons for PortKiller menu items
# Downloads SVGs from Devicon and converts to 32x32 PNG

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SOURCES_DIR="$PROJECT_DIR/assets/process-icons/sources"
GENERATED_DIR="$PROJECT_DIR/assets/process-icons/generated"

# Devicon base URL (raw GitHub content)
DEVICON_BASE="https://raw.githubusercontent.com/devicons/devicon/master/icons"

echo "Creating directories..."
mkdir -p "$SOURCES_DIR"
mkdir -p "$GENERATED_DIR"

echo "Downloading Devicon SVGs..."

# Download each icon
download_icon() {
    local name=$1
    local path=$2
    local url="$DEVICON_BASE/$path"
    local output="$SOURCES_DIR/${name}.svg"

    if [ ! -f "$output" ]; then
        echo "  Downloading $name..."
        curl -sL "$url" -o "$output"
    else
        echo "  $name already exists, skipping"
    fi
}

download_icon "nodejs" "nodejs/nodejs-original.svg"
download_icon "python" "python/python-original.svg"
download_icon "ruby" "ruby/ruby-original.svg"
download_icon "go" "go/go-original.svg"
download_icon "rust" "rust/rust-original.svg"
download_icon "java" "java/java-original.svg"
download_icon "php" "php/php-original.svg"
download_icon "postgresql" "postgresql/postgresql-original.svg"
download_icon "mysql" "mysql/mysql-original.svg"
download_icon "mongodb" "mongodb/mongodb-original.svg"
download_icon "redis" "redis/redis-original.svg"
download_icon "docker" "docker/docker-original.svg"

# Download Homebrew icon from SimpleIcons
echo "  Downloading homebrew..."
if [ ! -f "$SOURCES_DIR/homebrew.svg" ]; then
    curl -sL "https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/homebrew.svg" -o "$SOURCES_DIR/homebrew.svg"
fi

# Create generic terminal icon
echo "  Creating generic icon..."
cat > "$SOURCES_DIR/generic.svg" << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#666666">
  <rect x="2" y="3" width="20" height="18" rx="2" fill="none" stroke="#666666" stroke-width="1.5"/>
  <path d="M6 8l4 4-4 4" stroke="#666666" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
  <line x1="12" y1="16" x2="18" y2="16" stroke="#666666" stroke-width="1.5" stroke-linecap="round"/>
</svg>
EOF

echo ""
echo "Converting SVGs to PNG (32x32 @2x)..."

# Check for rsvg-convert
if ! command -v rsvg-convert &> /dev/null; then
    echo "Error: rsvg-convert not found. Install with: brew install librsvg"
    exit 1
fi

for svg in "$SOURCES_DIR"/*.svg; do
    name=$(basename "$svg" .svg)
    output="$GENERATED_DIR/${name}@2x.png"

    echo "  Converting $name..."
    rsvg-convert -w 32 -h 32 "$svg" -o "$output"
done

# Optimize PNGs if pngquant is available
if command -v pngquant &> /dev/null; then
    echo ""
    echo "Optimizing PNGs with pngquant..."
    for png in "$GENERATED_DIR"/*.png; do
        pngquant --quality=65-80 --ext .png --force "$png" 2>/dev/null || true
    done
else
    echo ""
    echo "Note: pngquant not found, skipping optimization"
fi

echo ""
echo "Done! Generated icons:"
ls -lh "$GENERATED_DIR"

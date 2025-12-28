# Development Setup Guide

Complete guide for setting up the Daydream Initiative development environment on Windows, macOS, and Linux.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Windows Setup](#windows-setup)
- [macOS/Linux Setup](#macoslinux-setup)
- [Database Configuration](#database-configuration)
- [IDE Setup](#ide-setup)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software

1. **Rust Toolchain** (1.70+)

   ```bash
   # Install rustup (Rust installer)
   # Windows: Download from https://rustup.rs/
   # macOS/Linux:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **PostgreSQL** (14+)
   - **Windows**: Download installer from <https://www.postgresql.org/download/windows/>
   - **macOS**: `brew install postgresql@14`
   - **Linux**: `sudo apt-get install postgresql-14` (Ubuntu/Debian)

3. **Trunk** (WASM bundler)

   ```bash
   cargo install trunk
   ```

4. **sqlx-cli** (Database migrations)

   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

### Optional (Recommended)

- **VS Code** with extensions:
  - rust-analyzer
  - CodeLLDB (debugging)
  - Even Better TOML
  - crates (dependency management)

---

## Windows Setup

### 1. Install Rust

Download and run `rustup-init.exe` from <https://rustup.rs/>

During installation, accept the default options.

### 2. Install PostgreSQL

1. Download PostgreSQL 14+ installer
2. Run installer, set password for `postgres` user
3. Remember the port (default: 5432)
4. Add PostgreSQL bin to PATH:

   ```powershell
   $env:Path += ";C:\Program Files\PostgreSQL\14\bin"
   ```

### 3. Configure Database

```powershell
# Set environment variable (permanent)
[System.Environment]::SetEnvironmentVariable(
    'DATABASE_URL',
    'postgres://postgres:YOUR_PASSWORD@localhost:5432/daydream',
    [System.EnvironmentVariableTarget]::User
)

# Or set for current session only
$env:DATABASE_URL = "postgres://postgres:YOUR_PASSWORD@localhost:5432/daydream"
```

### 4. Create Database

```powershell
cd backend
sqlx database create
sqlx migrate run
```

### 5. Run Project

```powershell
# Terminal 1: Backend
cd backend
cargo run

# Terminal 2: Frontend
cd frontend
trunk serve
```

---

## macOS/Linux Setup

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install PostgreSQL

**macOS (Homebrew)**:

```bash
brew install postgresql@14
brew services start postgresql@14
```

**Ubuntu/Debian**:

```bash
sudo apt-get update
sudo apt-get install postgresql-14 postgresql-contrib
sudo systemctl start postgresql
```

**Arch Linux**:

```bash
sudo pacman -S postgresql
sudo systemctl start postgresql
```

### 3. Configure PostgreSQL User

```bash
# Switch to postgres user
sudo -u postgres psql

# In psql shell, create user and database
CREATE USER daydream_user WITH PASSWORD 'secure_password';
CREATE DATABASE daydream OWNER daydream_user;
\q
```

### 4. Set Environment Variable

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export DATABASE_URL="postgres://daydream_user:secure_password@localhost:5432/daydream"
```

Then reload:

```bash
source ~/.bashrc  # or ~/.zshrc
```

### 5. Run Migrations

```bash
cd backend
sqlx migrate run
```

### 6. Run Project

```bash
# Terminal 1: Backend
cd backend
cargo run

# Terminal 2: Frontend  
cd frontend
trunk serve
```

---

## Database Configuration

### Production Environment Variables

Create a `.env` file in the `backend/` directory:

```env
DATABASE_URL=postgres://username:password@localhost:5432/daydream
RUST_LOG=info
PORT=8080
```

**Security Note**: Never commit `.env` files to version control! Add to `.gitignore`.

### Database Migration Workflow

```bash
# Create new migration
cd backend
sqlx migrate add create_new_table

# Edit the generated .sql file in backend/migrations/

# Apply migration
sqlx migrate run

# Revert last migration (if needed)
sqlx migrate revert
```

### Verify Database Connection

```bash
# Test connection
psql $DATABASE_URL

# List tables
\dt

# View schema
\d story_graphs
```

---

## IDE Setup

### VS Code (Recommended)

1. Install extensions:
   - `rust-analyzer` (Rust language server)
   - `CodeLLDB` (Debugging)
   - `Even Better TOML`
   - `crates` (Cargo.toml helper)

2. Create `.vscode/settings.json`:

   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy",
     "editor.formatOnSave": true,
     "[rust]": {
       "editor.defaultFormatter": "rust-lang.rust-analyzer"
     }
   }
   ```

3. Create `.vscode/launch.json` for debugging:

   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug Backend",
         "cargo": {
           "args": ["build", "--bin=backend", "--package=backend"],
           "filter": {
             "name": "backend",
             "kind": "bin"
           }
         },
         "args": [],
         "cwd": "${workspaceFolder}/backend"
       }
     ]
   }
   ```

### RustRover / IntelliJ IDEA

1. Install Rust plugin
2. Open project root as Cargo workspace
3. Enable "Run Clippy on save" in settings

---

## Troubleshooting

### "sqlx-data.json not found"

This error occurs when `cargo` can't verify SQL queries at compile time.

**Solution**:

```bash
# Option 1: Run migrations first
cd backend
sqlx migrate run

# Option 2: Enable offline mode
cargo sqlx prepare
# This generates sqlx-data.json for offline compilation
```

### "Port 8080 already in use"

**Solution**:

```bash
# Find process using port
# Windows:
netstat -ano | findstr :8080
taskkill /PID <PID> /F

# macOS/Linux:
lsof -i :8080
kill -9 <PID>

# Or change port in backend/.env
PORT=8081
```

### WASM Build Fails

```bash
# Ensure wasm target is installed
rustup target add wasm32-unknown-unknown

# Clear trunk cache
trunk clean

# Rebuild
trunk build
```

### PostgreSQL Connection Refused

**Check if PostgreSQL is running**:

```bash
# Windows:
Get-Service postgresql*

# macOS:
brew services list

# Linux:
sudo systemctl status postgresql
```

**Start if stopped**:

```bash
# Windows: Use Services app

# macOS:
brew services start postgresql@14

# Linux:
sudo systemctl start postgresql
```

### Rust Compilation is Slow

**Enable faster linker**:

**macOS/Linux** - Add to `~/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Windows** - Use `rust-lld` (already included):

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Frontend Hot Reload Not Working

```bash
# Clear browser cache
# Chrome/Edge: Ctrl+Shift+R
# Firefox: Ctrl+F5

# Restart trunk with clean build
trunk clean
trunk serve
```

---

## Verification Checklist

After setup, verify everything works:

- [ ] `rustc --version` shows 1.70+
- [ ] `cargo --version` works
- [ ] `trunk --version` works  
- [ ] `sqlx --version` works
- [ ] `psql $DATABASE_URL` connects successfully
- [ ] `cd backend && cargo run` starts server on port 8080
- [ ] `cd frontend && trunk serve` opens browser to <http://127.0.0.1:8080>
- [ ] Can save and load story graphs via UI

---

## Next Steps

Once setup is complete:

1. Read [README.md](README.md) for architecture overview
2. Review API documentation in main README
3. Check `backend/migrations/` for database schema
4. Explore `frontend/src/pages/` for UI components
5. Run tests: `cargo test --workspace`

---

## Getting Help

- **Rust Issues**: <https://users.rust-lang.org/>
- **PostgreSQL**: <https://www.postgresql.org/support/>
- **Leptos**: <https://book.leptos.dev/>
- **Project Issues**: Open issue on GitHub repository

---

**Happy Coding! 🦀**

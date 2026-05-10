const { spawn } = require('child_process');
const path = require('path');

const rust = spawn(path.join(__dirname, 'target/release/rust_vhproject'), [], {
  stdio: 'inherit'
});

rust.on('error', (err) => {
  console.error('Error starting Rust server:', err);
});

process.on('SIGINT', () => {
  rust.kill();
  process.exit();
});

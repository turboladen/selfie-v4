#!/usr/bin/env node
// JSON-RPC/stdio driver for selfie-mcp, plus a sandbox-HOME creator shared
// with `just sandbox-run` (the CLI's own driver). See SKILL.md for usage.

import { spawn } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import readline from "node:readline";

function sandboxEnv(home) {
  return {
    PATH: process.env.PATH,
    HOME: home,
    XDG_CONFIG_HOME: join(home, ".config"),
    SELFIE_CONFIG_DIR: join(home, ".config", "selfie"),
    SHELL: "/bin/sh",
    TERM: "dumb",
  };
}

function makeSandbox() {
  const home = mkdtempSync(join(tmpdir(), "selfie-mcp-sandbox."));
  const packageDir = join(home, "packages");
  const configDir = join(home, ".config", "selfie");
  mkdirSync(packageDir, { recursive: true });
  mkdirSync(configDir, { recursive: true });
  writeFileSync(
    join(configDir, "config.yaml"),
    `environment: sandbox\npackage_directory: ${packageDir}\n`,
  );
  writeFileSync(
    join(packageDir, "sandbox-sentinel.yaml"),
    'name: sandbox-sentinel\nenvironments:\n  sandbox:\n    install: "true"\n    check: "true"\n',
  );
  return home;
}

// Drives one selfie-mcp process over newline-delimited JSON-RPC (the rmcp
// stdio transport: no Content-Length framing, one JSON value per line).
class McpClient {
  constructor(home, binPath) {
    this.proc = spawn(binPath, [], { env: sandboxEnv(home), stdio: ["pipe", "pipe", "pipe"] });
    this.rl = readline.createInterface({ input: this.proc.stdout });
    this.nextId = 1;
    this.pending = new Map();
    this.stderr = "";
    // Unhandled 'error'/'exit' on a ChildProcess or a Writable crashes the
    // whole driver with a raw stack trace instead of a catchable rejection —
    // spawn() emits 'error' asynchronously (e.g. ENOENT when binPath hasn't
    // been built yet), and a dead process makes any further stdin write EPIPE.
    this.proc.on("error", (err) => {
      this._rejectAll(new Error(`failed to run ${binPath}: ${err.message}`));
    });
    this.proc.on("exit", (code, signal) => {
      this._rejectAll(
        new Error(
          `${binPath} exited (code ${code}, signal ${signal}) before responding; stderr so far:\n${this.stderr}`,
        ),
      );
    });
    this.proc.stdin.on("error", () => {});
    this.proc.stderr.on("data", (d) => {
      this.stderr += d.toString();
    });
    this.rl.on("line", (line) => {
      if (!line.trim()) return;
      let msg;
      try {
        msg = JSON.parse(line);
      } catch {
        this._rejectAll(new Error(`${binPath} wrote a non-JSON line on stdout: ${line}`));
        return;
      }
      if (msg.id !== undefined && this.pending.has(msg.id)) {
        this.pending.get(msg.id).resolve(msg);
        this.pending.delete(msg.id);
      }
    });
  }

  _rejectAll(err) {
    for (const { reject } of this.pending.values()) {
      reject(err);
    }
    this.pending.clear();
  }

  request(method, params) {
    const id = this.nextId++;
    const payload = { jsonrpc: "2.0", id, method, params };
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`timed out waiting for ${method}; stderr so far:\n${this.stderr}`));
        }
      }, 15000);
      // Wrap resolve/reject so the dangling timer is cleared on the fast path —
      // otherwise the process (and anything waiting on it) sits idle for the
      // rest of the 15s even after the response already arrived.
      this.pending.set(id, {
        resolve: (msg) => {
          clearTimeout(timer);
          resolve(msg);
        },
        reject: (err) => {
          clearTimeout(timer);
          reject(err);
        },
      });
      this.proc.stdin.write(JSON.stringify(payload) + "\n");
    });
  }

  notify(method, params) {
    this.proc.stdin.write(JSON.stringify({ jsonrpc: "2.0", method, params }) + "\n");
  }

  async initialize() {
    const response = await this.request("initialize", {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "selfie-run-skill-driver", version: "0.1" },
    });
    if (response.error) {
      throw new Error(`initialize failed: ${JSON.stringify(response.error)}`);
    }
    this.notify("notifications/initialized", {});
    return response;
  }

  async listTools() {
    return this.request("tools/list", {});
  }

  async callTool(name, args) {
    return this.request("tools/call", { name, arguments: args });
  }

  close() {
    this.rl.close();
    if (this.proc.exitCode !== null || this.proc.signalCode !== null) {
      return;
    }
    // SIGTERM asks nicely; escalate so a process that ignores it can't keep
    // the driver from returning control to the shell.
    return new Promise((resolve) => {
      const escalate = setTimeout(() => this.proc.kill("SIGKILL"), 2000);
      this.proc.once("exit", () => {
        clearTimeout(escalate);
        resolve();
      });
      this.proc.kill();
    });
  }
}

async function main() {
  const [cmd, ...rest] = process.argv.slice(2);

  if (cmd === "sandbox") {
    console.log(makeSandbox());
    return;
  }

  if (cmd === "list-tools" || cmd === "call") {
    const home = rest[0];
    const binPath = process.env.SELFIE_MCP_BIN ?? "./target/release/selfie-mcp";
    if (!home) {
      console.error(`usage: driver.mjs ${cmd} <sandbox-home> ...`);
      process.exit(2);
    }
    const client = new McpClient(home, binPath);
    try {
      await client.initialize();
      if (cmd === "list-tools") {
        const { result, error } = await client.listTools();
        if (error) {
          console.error(JSON.stringify(error, null, 2));
          process.exitCode = 1;
        } else {
          for (const tool of result.tools) {
            console.log(tool.name, "-", tool.description.split("\n")[0]);
          }
        }
      } else {
        const [toolName, argsJson] = rest.slice(1);
        if (!toolName) {
          console.error("usage: driver.mjs call <sandbox-home> <tool-name> [json-args]");
          process.exit(2);
        }
        const args = argsJson ? JSON.parse(argsJson) : {};
        const { result, error } = await client.callTool(toolName, args);
        if (error) {
          console.error(JSON.stringify(error, null, 2));
          process.exitCode = 1;
        } else {
          for (const block of result.content) {
            console.log(block.type === "text" ? block.text : JSON.stringify(block));
          }
          if (result.isError) process.exitCode = 1;
        }
      }
    } finally {
      await client.close();
    }
    return;
  }

  console.error("usage: driver.mjs sandbox | list-tools <home> | call <home> <tool> [json-args]");
  process.exit(2);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

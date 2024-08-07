const pg = require("pg");
const express = require("express");

const app = express();

const { POSTGRES_URL, NODE_ENV } = process.env;

if (!POSTGRES_URL) {
  throw new Error("POSTGRES_URL is not set");
}

console.log(`Running in ${NODE_ENV} mode.`);

const pool = new pg.Pool({
  connectionString: POSTGRES_URL,
});

async function main() {
  while (true) {
    try {
      await pool.connect();
      break;
    } catch (error) {
      console.log("Could not connect to DB, retrying in 1000 ms", error);
      await new Promise((res) => setTimeout(res, 1000));
    }
  }

  console.log("Connected to DB successfully");

  await pool.query(`
    CREATE TABLE IF NOT EXISTS users (
      id SERIAL PRIMARY KEY,
      name VARCHAR(255) NOT NULL
    )
  `);

  app.use(express.json());

  app.get("/", async (_, res) => {
    const { rows } = await pool.query("SELECT * FROM users");
    res.json({ rows, count: rows.length });
  });

  app.get("/insert", async (req, res) => {
    await pool.query(`
      INSERT INTO users (name)
      VALUES ('${req.query.name}')
    `);

    res.send("OK");
  });

  app.listen(3000, () => {
    console.log("Server started on port 3000");
  });

  let logNumber = 0;

  setInterval(() => {
    console.log(logNumber++);
  }, 1000);
}

void main();

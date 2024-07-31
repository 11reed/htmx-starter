import express from 'express';
import bodyParser from 'body-parser';
import dotenv from 'dotenv';
import { createClient } from '@libsql/client';
import { join } from 'path';
import nunjucks from 'nunjucks';
import Timeout from 'await-timeout';

dotenv.config();

const app = express();
const port = 3000;

// Configure Nunjucks
nunjucks.configure(join(__dirname, '../templates'), {
  autoescape: true,
  express: app,
  watch: true
});

// Database setup
const turso = createClient({
  url: process.env.LIBSQL_URL as string,
  authToken: process.env.LIBSQL_AUTH_TOKEN as string,
});

interface Post {
  id: number;
  title: string;
  content?: string;
}

app.use(bodyParser.urlencoded({ extended: true }));
app.use('/static', express.static(join(__dirname, '../static')));
app.use('/css', express.static(join(__dirname, '../src/css')));

// Route handlers
app.get('/', async (req, res) => {
  try {
    const result = await turso.execute('SELECT id, title, content FROM posts');
    const posts: Post[] = result.rows.map((row: any) => ({
      id: row[0],
      title: row[1],
      content: row[2],
    }));

    res.render('index.njk', { posts });
  } catch (err) {
    console.error('Failed to execute query:', err);
    res.sendStatus(500);
  }
});

app.post('/create_post', async (req, res) => {
  const { title, content } = req.body as { title: string; content?: string };
  try {
    await turso.execute({
      sql: 'INSERT INTO posts (title, content) VALUES (?, ?)',
      args: [title, content || ''],
    });

    const result = await turso.execute('SELECT id, title, content FROM posts');
    const posts: Post[] = result.rows.map((row: any) => ({
      id: row[0],
      title: row[1],
      content: row[2],
    }));

    res.render('index.njk', { posts });
  } catch (err) {
    console.error('Failed to execute query:', err);
    res.sendStatus(500);
  }
});

// Ensure the posts table exists
async function createTableIfNotExists() {
  await turso.execute(`
    CREATE TABLE IF NOT EXISTS posts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL,
      content TEXT
    );
  `);
}

// Keep the connection alive
async function keepAlive() {
  const timeout = new Timeout();
  try {
    while (true) {
      await turso.execute('SELECT 1');
      await timeout.set(300000); // Ping every 5 minutes
    }
  } catch (err) {
    console.error('Failed to keep connection alive:', err);
  }
}

// Start the server
(async () => {
  await createTableIfNotExists();
  keepAlive();

  app.listen(port, () => {
    console.log(`Server is listening on http://localhost:${port}`);
  });
})();


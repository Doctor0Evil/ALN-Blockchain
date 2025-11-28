/**
 * Simple static file server for ALN Explorer
 */

const express = require('express');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 8080;

// Serve static files
app.use(express.static(__dirname));

// Serve index.html for root
app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'index.html'));
});

app.listen(PORT, () => {
  console.log(`✅ ALN Explorer serving on http://localhost:${PORT}`);
  console.log(`📊 Open your browser to view the explorer`);
});

const https = require('https');
const fs = require('fs');

var args = process.argv;
var options = {
  hostname: args[2],
  port: args[3],
  path: args[4],
  method: 'POST',
  key: fs.readFileSync('../certs/client/user.key'),
  cert: fs.readFileSync('../certs/client/user.crt'),
  ca: fs.readFileSync('../certs/client/ca.crt'),
  rejectUnauthorized: false
};
console.log(options);
var req = https.request(options, (res) => {
  console.log('statusCode: ', res.statusCode);
  console.log('headers: ', res.headers);

  res.on('data', (d) => {
    process.stdout.write(d);
  });
});

// write data to request body
req.write(args[5]);
req.end();

req.on('error', (e) => {
  console.error(e);
});

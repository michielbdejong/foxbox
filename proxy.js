#!/usr/bin/env node
'use strict';

var https = require('https');
var fs = require('fs');
var qr = require('qr-image');
var exec = require('child_process').exec;
var proxy = require('http-proxy').createProxyServer();
var mdns = require('mdns');
var ports = {
  backend: 3000,
  front: 4333
};

function run(cmd) {
  return new Promise((resolve, reject) => {
    exec(cmd, (err, stdout, sterr) => {
      if (err) {
        reject(err);
      }
      resolve(stdout);
    });
  });
}

function build() {
  let fqdn;
  // Make directories to work from
  return run('mkdir -p certs/{server,client,ca,tmp}').then(() => {
    // Create your very own Root Certificate Authority
    return run('openssl genrsa -out certs/ca/my-root-ca.key.pem 2048');
  }).then(() => {
    // Self-sign your Root Certificate Authority
    // Since this is private, the details can be as bogus as you like
    return run('openssl req -x509 -new -nodes -key certs/ca/my-root-ca.key.pem ' +
        '-days 1000000 -out certs/ca/my-root-ca.crt.pem ' +
        '-subj "/C=US/ST=Utah/L=Provo/O=ACME Signing Authority Inc/CN=example.com"');
  }).then(() => {
    // Create a Device Certificate for each domain,
    // such as example.com, *.example.com, awesome.example.com
    // NOTE: You MUST match CN to the domain name or ip address you want to use
    return run('openssl genrsa -out certs/server/my-server.key.pem 2048');
  }).then(() => {
    // Determine the fingerprint of the signing cert
    return run('openssl x509 -in certs/ca/my-root-ca.crt.pem -sha256 -noout -fingerprint');
  }).then(out => {
    fqdn = out.substring('SHA256 Fingerprint='.length)
        .split(':').join('').toLowerCase().trim().substring(0, 32) + '.self-signed';
    // Create a request from your Device, which your Root CA will sign
    return run('openssl req -new -key certs/server/my-server.key.pem ' +
      '-out certs/tmp/my-server.csr.pem ' +
      `-subj "/C=US/ST=Utah/L=Provo/O=ACME Tech Inc/CN=${fqdn}"`);
  }).then(() => {
    // Sign the request from Device with your Root CA
    // -CAserial certs/ca/my-root-ca.srl
    return run('openssl x509 -req -in certs/tmp/my-server.csr.pem ' +
        '-CA certs/ca/my-root-ca.crt.pem ' +
        '-CAkey certs/ca/my-root-ca.key.pem ' +
        '-CAcreateserial ' +
        '-out certs/server/my-server.crt.pem ' +
        '-days 1000000');
  }).then(() => {
    console.log(`Generated certificate chain for ${fqdn} in ./certs.`);
    return fqdn;
  });
}

function mdnsServe(fqdn) {
  // advertise a https server:
  mdns.createAdvertisement(mdns.tcp('https'), ports.front, {
    // seems that https://www.npmjs.com/package/cordova-plugin-zeroconf does not
    // support custom name field, so using txtRecord instead:
    txtRecord: {
      name: fqdn
    }
  }).start();

  // // For debugging purposes:
  // var browser = mdns.createBrowser(mdns.tcp('https'));
  // browser.on('serviceUp', function(service) {
  //   console.log("service up: ", service);
  // });
  // browser.on('serviceDown', function(service) {
  //   console.log("service down: ", service);
  // });
  // browser.start();
}

function qrGen(fqdn) {
  const qrCodeString = `https://${fqdn}:${ports.front}/`;
  const qr_svg = qr.image(qrCodeString, { type: 'svg' });
  qr_svg.pipe(fs.createWriteStream('qr.svg'));
  console.log(`Wrote string ${qrCodeString} into ./qr.svg, please display and scan.`);
}

function proxyServe(fqdn) {
  // serve a web server on the local network:
  https.createServer({
    key: fs.readFileSync('certs/server/my-server.key.pem'),
    cert: fs.readFileSync('certs/server/my-server.crt.pem'),
    ca: fs.readFileSync('certs/ca/my-root-ca.crt.pem')
  }, (req, res) => {
    proxy.web(req, res, { target: `http://localhost:${ports.backend}` });
  }).listen(ports.front);
  console.log(`Proxying https port ${ports.front} to http port ${ports.backend}, ` +
      `ready for connections.`);
}

//...

build().then(fqdn => {
  mdnsServe(fqdn);
  qrGen(fqdn);
  proxyServe(fqdn);
}).catch(err => {
  console.error(err);
});
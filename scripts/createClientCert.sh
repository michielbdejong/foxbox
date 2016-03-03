openssl genrsa -des3 -out ca.key 2048
openssl req -new -x509 -days 365 -key ca.key -out ca.crt
openssl x509 -in ca.crt -text -noout

openssl genrsa -out user.key 1024
openssl req -new -key user.key -out user.csr
openssl x509 -req -in user.csr -out user.crt -CA ca.crt -CAkey ca.key -CAcreateserial -days 365
openssl x509 -in user.crt -text -noout

mkdir -p ../certs/client/
mv ca.crt ../certs/client/
mv user.key ../certs/client/
mv user.crt ../certs/client/
rm user.csr
rm ca.*

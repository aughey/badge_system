#!/bin/sh

CA_PASS=foobar
SERVER_PASS=bigbird
PKI_PASS=blech

# Generate a private key for the CA
openssl genrsa -passout pass:$CA_PASS -des3 -out CA_private.key 2048

# Generate root certificate
openssl req -passin pass:$CA_PASS -x509 -new -nodes -key CA_private.key -sha256 -days 1024 -out CA_cert.crt -subj "/C=US/ST=CA/L=San Francisco/O=My Company/OU=IT Department/CN=My Company CA"

# Generate a server certificate signed by CA
openssl genrsa -passout pass:$SERVER_PASS -des3 -out server.key 2048

# Create a CSR (Certificate Signing Request) for a server cert
openssl req -passin pass:$SERVER_PASS -new -key server.key -out server.csr -subj "/C=US/ST=CA/L=San Francisco/O=My Company/OU=IT Department/CN=server.mycompany.com"

# Sign the CSR with the CA
openssl x509 -passin pass:$CA_PASS -req -in server.csr -CA CA_cert.crt -CAkey CA_private.key -CAcreateserial -out server.crt -days 500 -sha256

# Create a CSR (Certificate Signing Request) for a pki cert
openssl req -passout pass:$PKI_PASS -new -keyout pki_private.key -out pki_cert.csr -subj "/C=US/ST=CA/L=San Francisco/O=My Company/OU=IT Department/CN=pki.mycompany.com"

# Sign the CSR with the CA
openssl x509 -passin pass:$CA_PASS -req -in pki_cert.csr -CA CA_cert.crt -CAkey CA_private.key -CAcreateserial -out pki_cert.crt -days 500 -sha256

# Create a crl (empty)
openssl ca -passin pass:$CA_PASS -gencrl -keyfile CA_private.key -cert CA_cert.crt -out CA_crl.pem
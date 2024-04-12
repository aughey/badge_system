#!/bin/sh

# https://evilshit.wordpress.com/2013/06/19/how-to-create-your-own-pki-with-openssl/
rm *.crt *.csr *.key *.srl

CA_PASS=foobar
PKI_PASS=blech

# Generate a private key for the CA
echo "Generating a private key for the CA"
openssl genrsa -passout pass:$CA_PASS -des3 -out CA_private.key 2048

# Generate root certificate
echo "Generating a root certificate"
openssl req -passin pass:$CA_PASS -x509 -new -nodes -key CA_private.key -sha256 -days 1024 -out CA_cert.crt -subj "/C=US/ST=CA/L=Saint Louis/O=ThinkSplat/OU=IT Department/CN=ThinkSplat CA"

# Generate a server certificate signed by CA
echo "Generating a server private key"
openssl genrsa -out server.key 2048

# Create a CSR (Certificate Signing Request) for a server cert
echo "Creating a CSR for the server"
openssl req -new -key server.key -out server.csr -subj "/C=US/ST=CA/L=Saint Louis/O=ThinkSplat/OU=IT Department/CN=server.mycompany.com"

# Sign the cerver CSR with the CA
echo "Signing the server CSR with the CA"
openssl x509 -passin pass:$CA_PASS -req -in server.csr -CA CA_cert.crt -CAkey CA_private.key -CAcreateserial -out server.crt -days 500 -sha256

# Generate a client key (embedded_tls needs ec key)
echo "Generating a client private key"
openssl ecparam -genkey -name prime256v1 -noout -out client.key

# Create a CSR (Certificate Signing Request) for a pki cert
echo "Creating a CSR for the client"
openssl req -new -key client.key -out client.csr -subj "/C=US/ST=CA/L=Saint Louis/O=ThinkSplat/OU=IT Department/CN=pki.mycompany.com" -addext "subjectAltName=DNS:pki.mycompany.com"

# Sign the CSR with the CA
echo "Signing the client CSR with the CA"
openssl x509 -passin pass:$CA_PASS -req -in client.csr -extfile v3.ext -CA CA_cert.crt -CAkey CA_private.key -CAcreateserial -out client.crt -days 500 -sha256

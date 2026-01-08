# P2P TLS Setup Guide

## Overview

This guide explains how to configure TLS encryption for peer-to-peer (P2P) connections in the Ouroboros network. TLS provides:

- **Confidentiality**: P2P messages are encrypted and cannot be intercepted
- **Authentication**: Peers verify each other's identity using certificates
- **Integrity**: Messages cannot be tampered with in transit
- **Replay protection**: Built-in protection against replay attacks

## Prerequisites

- OpenSSL installed (`openssl version`)
- Access to the node's filesystem
- Understanding of TLS certificates and PKI

## Quick Start

For production deployments, we recommend using proper Certificate Authority (CA) signed certificates. For testing/development, you can use self-signed certificates.

### Option 1: Production Setup (CA-Signed Certificates)

#### Step 1: Generate Certificate Signing Request (CSR)

```bash
# Generate private key (keep this secret!)
openssl genpkey -algorithm RSA -out node_private_key.pem -pkeyopt rsa_keygen_bits:2048

# Generate CSR
openssl req -new -key node_private_key.pem -out node_csr.pem \
  -subj "/C=US/ST=State/L=City/O=OrganizationName/OU=Ouroboros/CN=node1.example.com"
```

#### Step 2: Submit CSR to Certificate Authority

Submit `node_csr.pem` to your CA (e.g., Let's Encrypt, DigiCert, or internal CA).

#### Step 3: Install Certificates

Once you receive the signed certificate and CA chain:

```bash
# Copy files to secure location
cp node_private_key.pem /etc/ouroboros/tls/key.pem
cp signed_certificate.pem /etc/ouroboros/tls/cert.pem
cp ca_chain.pem /etc/ouroboros/tls/ca.pem

# Set strict permissions
chmod 600 /etc/ouroboros/tls/key.pem
chmod 644 /etc/ouroboros/tls/cert.pem
chmod 644 /etc/ouroboros/tls/ca.pem
```

#### Step 4: Configure Node

Add to `.env`:

```bash
TLS_CERT_PATH=/etc/ouroboros/tls/cert.pem
TLS_KEY_PATH=/etc/ouroboros/tls/key.pem
TLS_CA_CERT_PATH=/etc/ouroboros/tls/ca.pem
```

### Option 2: Development Setup (Self-Signed Certificates)

**WARNING**: Self-signed certificates should NEVER be used in production!

#### Generate Self-Signed Certificate

```bash
# Create directory
mkdir -p tls_test

# Generate self-signed certificate (valid for 365 days)
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout tls_test/key.pem \
  -out tls_test/cert.pem \
  -days 365 \
  -subj "/C=US/ST=Dev/L=Local/O=Test/CN=localhost"

# View certificate details
openssl x509 -in tls_test/cert.pem -text -noout
```

#### Configure Node

Add to `.env`:

```bash
TLS_CERT_PATH=tls_test/cert.pem
TLS_KEY_PATH=tls_test/key.pem
# No CA cert needed for self-signed (peer verification disabled in dev mode)
```

## Certificate Management

### Certificate Rotation

TLS certificates expire. Plan for rotation:

```bash
# Check certificate expiration
openssl x509 -in /etc/ouroboros/tls/cert.pem -noout -enddate

# Set up renewal cron job (example for Let's Encrypt)
0 0 1 * * certbot renew --deploy-hook "systemctl restart ouroboros"
```

### Multi-Node Setup

Each validator node should have its own certificate:

**Node 1:**
```bash
# .env
TLS_CERT_PATH=/etc/ouroboros/node1/cert.pem
TLS_KEY_PATH=/etc/ouroboros/node1/key.pem
```

**Node 2:**
```bash
# .env
TLS_CERT_PATH=/etc/ouroboros/node2/cert.pem
TLS_KEY_PATH=/etc/ouroboros/node2/key.pem
```

**Node 3:**
```bash
# .env
TLS_CERT_PATH=/etc/ouroboros/node3/cert.pem
TLS_KEY_PATH=/etc/ouroboros/node3/key.pem
```

### Certificate Verification

Enable peer certificate verification (production recommended):

```bash
# .env
TLS_VERIFY_PEER=true
TLS_CA_CERT_PATH=/etc/ouroboros/tls/ca.pem
```

Disable verification (development only):

```bash
# .env
TLS_VERIFY_PEER=false
```

## Security Considerations

### Key Security

1. **Private Key Protection**:
   - Store private keys with `600` permissions (owner read/write only)
   - Never commit private keys to version control
   - Use hardware security modules (HSM) for production
   - Rotate keys if compromised

2. **Certificate Pinning** (Advanced):
   ```bash
   # Pin specific peer certificates
   # .env
   TLS_PINNED_PEERS=node1:sha256/abc123...,node2:sha256/def456...
   ```

3. **TLS Version**:
   - Ouroboros uses TLS 1.3 by default (most secure)
   - Fallback to TLS 1.2 is supported but not recommended

### Firewall Configuration

Allow TLS connections through firewall:

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 9001/tcp comment "Ouroboros P2P TLS"
sudo ufw allow 9091/tcp comment "Ouroboros BFT TLS"

# iptables
sudo iptables -A INPUT -p tcp --dport 9001 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 9091 -j ACCEPT
```

## Troubleshooting

### Common Issues

#### 1. "TLS handshake failed"

**Cause**: Certificate mismatch or expired certificate

**Solution**:
```bash
# Check certificate validity
openssl x509 -in /etc/ouroboros/tls/cert.pem -noout -dates

# Verify certificate matches private key
openssl x509 -noout -modulus -in /etc/ouroboros/tls/cert.pem | openssl md5
openssl rsa -noout -modulus -in /etc/ouroboros/tls/key.pem | openssl md5
# Hashes should match
```

#### 2. "Permission denied" reading certificate

**Cause**: File permissions too restrictive or incorrect ownership

**Solution**:
```bash
# Fix ownership
sudo chown ouroboros:ouroboros /etc/ouroboros/tls/*.pem

# Fix permissions
chmod 600 /etc/ouroboros/tls/key.pem
chmod 644 /etc/ouroboros/tls/cert.pem
```

#### 3. "Certificate verification failed"

**Cause**: CA certificate not trusted or missing

**Solution**:
```bash
# Ensure CA certificate is configured
# .env
TLS_CA_CERT_PATH=/etc/ouroboros/tls/ca.pem

# Or disable verification for testing
TLS_VERIFY_PEER=false
```

#### 4. Viewing TLS handshake details

Enable debug logging:

```bash
# .env
RUST_LOG=debug

# Check logs
tail -f node.log | grep -i tls
```

### Testing TLS Connection

Test TLS connectivity with OpenSSL:

```bash
# Test connection to peer
openssl s_client -connect node1.example.com:9001 -CAfile /etc/ouroboros/tls/ca.pem

# Expected output:
# SSL-Session:
#   Protocol  : TLSv1.3
#   Cipher    : TLS_AES_256_GCM_SHA384
#   Verify return code: 0 (ok)
```

Test with specific TLS version:

```bash
# Force TLS 1.3
openssl s_client -connect localhost:9001 -tls1_3

# Force TLS 1.2
openssl s_client -connect localhost:9001 -tls1_2
```

## Performance Considerations

### TLS Overhead

TLS adds ~5-10% CPU overhead and ~1-2ms latency per connection. For high-throughput deployments:

1. **Use AES-NI hardware acceleration** (enabled by default on modern CPUs)
2. **Connection reuse**: Keep connections alive (Ouroboros does this automatically)
3. **Batch messages**: Reduce handshake overhead by batching when possible

### Benchmarking

Measure TLS impact:

```bash
# Without TLS (baseline)
time curl http://localhost:8001/health

# With TLS
time curl https://localhost:8001/health
```

## Production Checklist

- [ ] Use CA-signed certificates (not self-signed)
- [ ] Enable peer certificate verification (`TLS_VERIFY_PEER=true`)
- [ ] Set up certificate rotation/renewal
- [ ] Configure firewall rules
- [ ] Set strict file permissions (600 for private keys)
- [ ] Never commit private keys to version control
- [ ] Monitor certificate expiration
- [ ] Test failover scenarios
- [ ] Document certificate locations in runbooks
- [ ] Use TLS 1.3 (disable TLS 1.2 fallback if possible)

## Compliance

### FIPS 140-2 Compliance

For FIPS-compliant deployments:

```bash
# Use FIPS-approved algorithms
TLS_CIPHERS=TLS_AES_256_GCM_SHA384:TLS_AES_128_GCM_SHA256
TLS_MIN_VERSION=1.3
```

### PCI DSS Requirements

PCI DSS 4.0 requirements for TLS:
- ✅ TLS 1.3 supported
- ✅ Strong cipher suites (AES-256-GCM)
- ✅ Certificate validation
- ✅ Key rotation support

## Advanced: Mutual TLS (mTLS)

For maximum security, require client certificates:

```bash
# .env
TLS_REQUIRE_CLIENT_CERT=true
TLS_CLIENT_CA_CERT_PATH=/etc/ouroboros/tls/client_ca.pem
```

Each connecting peer must present a valid client certificate signed by the trusted CA.

## References

- [RFC 8446: TLS 1.3](https://tools.ietf.org/html/rfc8446)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [OWASP TLS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html)

## Support

For issues related to TLS configuration:
1. Check logs: `grep -i tls node.log`
2. Verify certificate chain: `openssl verify -CAfile ca.pem cert.pem`
3. Report issues: https://github.com/anthropics/ouroboros/issues

# Linux deployment with WAF and intrusion prevention

This deployment keeps the Rust app off public host ports and serves the UI and API from one origin (cross-origin API access is disabled). Host Nginx terminates TLS and uses the CrowdSec Nginx bouncer; the OWASP ModSecurity CRS container filters requests before they reach the app. CrowdSec and Fail2ban read the host Nginx access log, which records the client IP before proxying, and Fail2ban bans repeated failed sign-in attempts at the host firewall.

## 1. Prepare the host

Use a supported Linux VPS with Docker Engine and the Compose plugin. Point the domain's DNS A/AAAA records at the VPS. Install host Nginx, Certbot, and Fail2ban using your distribution's packages. Add the official CrowdSec package repository, but install its Nginx bouncer only after the Compose CrowdSec engine is running. The engine itself runs in Compose, so do not start a second host engine. The Nginx bouncer and Fail2ban run on the host so they can use the host Nginx module and firewall.

Obtain a TLS certificate before enabling the site configuration. For Debian/Ubuntu, one option is:

```sh
sudo apt update
sudo apt install nginx certbot python3-certbot-nginx fail2ban
sudo certbot --nginx -d YOUR_DOMAIN
```

Do not expose CrowdSec's Local API publicly; Compose binds it to `127.0.0.1:8080`.

## 2. Configure secrets and start containers

From the repository root:

```sh
cp .env.example .env
chmod 600 .env
```

Set `DOMAIN`, the Cloudflare Turnstile site/secret keys, `BREVO_API_KEY` to an API key for the Brevo account authorized to send from `rustcord1@gmail.com`, and a long random `CROWDSEC_BOUNCER_KEY` in `.env`. Generate the CrowdSec key with `openssl rand -hex 32`. Keep `.env` private and out of source control.

Start the app, ModSecurity/CRS, and CrowdSec:

```sh
docker compose up -d --build
```

The WAF is bound only to `127.0.0.1:8081`; the app has no published host port. Do not publish either port to the Internet.

RustCord sends email verification codes for registration and every new sign-in, and uses the same sender for password recovery. Codes expire after ten minutes, are limited to five attempts, and a new code can be requested once per minute for each address and purpose; the server also caps code sends at 30 per minute. Configure a valid Brevo API key before starting; sign-in, registration, and recovery requests that need email delivery fail explicitly when email delivery is unavailable. Existing sessions are revoked once when email verification is first enabled so old unverified sessions cannot bypass the new login check.

For local development, `start-local.ps1` reads `BREVO_API_KEY` as a secure prompt if it is not already set in the process environment.

## 3. Enable host Nginx, CrowdSec, and Fail2ban

Copy `deploy/nginx/rustcord-log-format.conf` to `/etc/nginx/conf.d/` and `deploy/nginx/rustcord.conf.example` to `/etc/nginx/sites-available/rustcord`. Replace every `YOUR_DOMAIN` with the domain from `.env`, enable the site, and check the configuration before reloading:

```sh
sudo ln -s /etc/nginx/sites-available/rustcord /etc/nginx/sites-enabled/rustcord
sudo nginx -t
sudo systemctl reload nginx
```

Install `crowdsec-nginx-bouncer` from the official CrowdSec repository. Configure `/etc/crowdsec/bouncers/crowdsec-nginx-bouncer.conf` with `API_URL=http://127.0.0.1:8080/` and the same `CROWDSEC_BOUNCER_KEY` from `.env`. Ensure the host Nginx bouncer package enables its `crowdsec_nginx.conf` Lua configuration, then validate and reload Nginx again.

Install the supplied Fail2ban filter and jail:

```sh
sudo install -m 0644 deploy/fail2ban/filter.d/rustcord-auth.conf /etc/fail2ban/filter.d/
sudo install -m 0644 deploy/fail2ban/jail.d/rustcord-auth.conf /etc/fail2ban/jail.d/
sudo fail2ban-client -t
sudo systemctl restart fail2ban
```

The jail bans an IP after ten `/api/auth` responses with status `401` in ten minutes, for one hour. Keep SSH protection configured separately and verify that your host firewall allows SSH before enabling any firewall bouncer.

## CSP maintenance

The enforced Content Security Policy pins the current inline application script by SHA-256. If the inline script in `Client/index.html` changes, run this from the repository root and replace the `sha256-...` value in the Nginx site configuration with the result:

```sh
node -e 'const fs=require("fs"),c=require("crypto"),s=[...fs.readFileSync("Client/index.html","utf8").matchAll(/<script(?:\s[^>]*)?>([\s\S]*?)<\/script>/gi)].filter(m=>!/\bsrc\s*=/.test(m[0].slice(0,m[0].indexOf(">"))));if(s.length!==1)throw Error(`Expected one inline script, found ${s.length}`);console.log("sha256-"+c.createHash("sha256").update(s[0][1].replace(/\r\n?/g,"\n")).digest("base64"))'
```

`script-src-attr 'unsafe-inline'` is retained because the current UI uses inline event-handler attributes; remove it only after migrating those handlers to JavaScript listeners.

## Verification and operations

Check `docker compose ps`, `docker compose logs waf crowdsec`, `sudo fail2ban-client status rustcord-auth`, and `sudo nginx -t` after deployment. Review ModSecurity and Fail2ban events before changing CRS rules or ban thresholds; do not disable CRS rules wholesale to work around false positives. Keep image versions updated and back up the `app-data`, `crowdsec-config`, and `crowdsec-data` volumes.

## Render free tier with Neon PostgreSQL

Render free web services do not retain local files across all restarts/redeploys. For persistent app data without a paid Render disk, create a free PostgreSQL project with a provider such as Neon, then add its full TLS-enabled connection string as `DATABASE_URL` in the Render service's Environment settings. Never commit the actual URL or password to GitHub. When `DATABASE_URL` is set, RustCord stores its full application state in PostgreSQL; otherwise it keeps using `APP_DATA_FILE` (local JSON). Existing local JSON is imported once only when the database table is empty. Keep a single app instance because the current application state is also held in memory, and make periodic exports/backups. A new empty Neon database cannot restore data that was already lost from a previous deployment.

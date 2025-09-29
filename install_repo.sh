#!/usr/bin/env bash
set -euo pipefail

REPO_URL=https://github.com/ipswyworld/ouroboros
# The final location of the code
CODE_DIR=/opt/ouroboros/ouro_dag
# A temporary directory for cloning
CLONE_DIR=/opt/ouroboros/ouroboros_clone
SERVICE_USER=ouro

# Ensure the service user exists
if ! id -u "${SERVICE_USER}" >/dev/null 2>&1; then
    echo "Creating system user ${SERVICE_USER}..."
    sudo useradd --system --create-home --shell /usr/sbin/nologin "${SERVICE_USER}"
fi

echo "1) prepare dirs & permissions"
sudo mkdir -p "${CODE_DIR}"
sudo mkdir -p "${CLONE_DIR}"
sudo chown -R "${SERVICE_USER}:${SERVICE_USER}" /opt/ouroboros

echo "2) clone or update repo"
# If the final code directory already has a .git folder, we assume it's correctly set up and just pull.
if [ -d "${CODE_DIR}/.git" ]; then
  echo "Git repository exists in final location, pulling latest changes."
  sudo -u "${SERVICE_USER}" git -C "${CODE_DIR}" pull --ff-only
else
  echo "Cloning fresh repository into temporary directory."
  # Clone the repo into a temporary location
  sudo -u "${SERVICE_USER}" git clone "${REPO_URL}" "${CLONE_DIR}"

  echo "Moving contents from nested ouro_dag directory to final location."
  # Move the contents of the nested ouro_dag directory to the final location
  sudo -u "${SERVICE_USER}" rsync -a "${CLONE_DIR}/ouro_dag/" "${CODE_DIR}/"

  echo "Cleaning up temporary clone directory."
  sudo rm -rf "${CLONE_DIR}"
fi

echo "3) ensure ownership + data dirs"
sudo chown -R "${SERVICE_USER}:${SERVICE_USER}" "${CODE_DIR}"
sudo -u "${SERVICE_USER}" mkdir -p "${CODE_DIR}/sled_data"
sudo -u "${SERVICE_USER}" mkdir -p "${CODE_DIR}/rocksdb_data"

echo "4) make migration script executable (if present)"
if [ -f "${CODE_DIR}/scripts/run_migrations.sh" ]; then
    sudo chmod +x "${CODE_DIR}/scripts/run_migrations.sh"
fi

echo "5) show repo top-level and migrations/scripts presence"
ls -la "${CODE_DIR}" | sed -n '1,120p'
test -d "${CODE_DIR}/migrations" && echo "migrations: OK" || echo "migrations: MISSING"
test -d "${CODE_DIR}/scripts" && echo "scripts: OK" || echo "scripts: MISSING"

echo "Installation script finished."
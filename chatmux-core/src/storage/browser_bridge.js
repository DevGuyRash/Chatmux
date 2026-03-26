const DB_VERSION = 1;

function globalApi(name) {
  return globalThis[name] ?? null;
}

function extensionStorageLocal() {
  const browserApi = globalApi("browser");
  if (browserApi?.storage?.local) return { area: browserApi.storage.local, style: "promise" };

  const chromeApi = globalApi("chrome");
  if (chromeApi?.storage?.local) return { area: chromeApi.storage.local, style: "callback" };

  throw new Error("extension storage.local is unavailable");
}

function requestToPromise(request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB request failed"));
  });
}

function transactionDone(transaction) {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error ?? new Error("IndexedDB transaction failed"));
    transaction.onabort = () => reject(transaction.error ?? new Error("IndexedDB transaction aborted"));
  });
}

function openDb(dbName, storeNames) {
  const indexedDb = globalThis.indexedDB;
  if (!indexedDb) {
    throw new Error("indexedDB is unavailable");
  }

  return new Promise((resolve, reject) => {
    const request = indexedDb.open(dbName, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      for (const storeName of storeNames) {
        if (!db.objectStoreNames.contains(storeName)) {
          db.createObjectStore(storeName, { keyPath: "id" });
        }
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB open failed"));
  });
}

export async function idb_put(dbName, storeName, storeNames, value) {
  const db = await openDb(dbName, storeNames);
  try {
    const transaction = db.transaction(storeName, "readwrite");
    transaction.objectStore(storeName).put(value);
    await transactionDone(transaction);
    return null;
  } finally {
    db.close();
  }
}

export async function idb_get_all(dbName, storeName, storeNames) {
  const db = await openDb(dbName, storeNames);
  try {
    const transaction = db.transaction(storeName, "readonly");
    const result = await requestToPromise(transaction.objectStore(storeName).getAll());
    await transactionDone(transaction);
    return result;
  } finally {
    db.close();
  }
}

export async function idb_get(dbName, storeName, storeNames, key) {
  const db = await openDb(dbName, storeNames);
  try {
    const transaction = db.transaction(storeName, "readonly");
    const result = await requestToPromise(transaction.objectStore(storeName).get(key));
    await transactionDone(transaction);
    return result ?? undefined;
  } finally {
    db.close();
  }
}

export async function idb_delete(dbName, storeName, storeNames, key) {
  const db = await openDb(dbName, storeNames);
  try {
    const transaction = db.transaction(storeName, "readwrite");
    transaction.objectStore(storeName).delete(key);
    await transactionDone(transaction);
    return null;
  } finally {
    db.close();
  }
}

export async function storage_local_get(key) {
  const { area, style } = extensionStorageLocal();
  if (style === "promise") {
    const result = await area.get(key);
    return result?.[key];
  }

  return new Promise((resolve, reject) => {
    area.get(key, (result) => {
      const runtimeError = globalThis.chrome?.runtime?.lastError;
      if (runtimeError) {
        reject(new Error(runtimeError.message || String(runtimeError)));
        return;
      }
      resolve(result?.[key]);
    });
  });
}

export async function storage_local_set(key, value) {
  const { area, style } = extensionStorageLocal();
  const payload = { [key]: value };
  if (style === "promise") {
    await area.set(payload);
    return null;
  }

  return new Promise((resolve, reject) => {
    area.set(payload, () => {
      const runtimeError = globalThis.chrome?.runtime?.lastError;
      if (runtimeError) {
        reject(new Error(runtimeError.message || String(runtimeError)));
        return;
      }
      resolve(null);
    });
  });
}

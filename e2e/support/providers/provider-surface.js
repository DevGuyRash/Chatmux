const crypto = require("node:crypto");

const SAFE_TEXT_LIMIT = 500;

function normalizeText(value) {
  return String(value || "").replace(/\s+/g, " ").trim();
}

function safeText(value, limit = SAFE_TEXT_LIMIT) {
  const normalized = normalizeText(value)
    .replace(/Bearer\s+[A-Za-z0-9._~+/=-]+/gi, "Bearer [redacted]")
    .replace(/(?:sk|key|token)[-_][A-Za-z0-9_-]{12,}/gi, "[redacted-token]");
  return normalized.length > limit
    ? `${normalized.slice(0, limit)}…`
    : normalized;
}

function safeUrl(value) {
  try {
    const url = new URL(value);
    url.search = "";
    url.hash = "";
    return url.toString();
  } catch {
    return safeText(value, 300);
  }
}

function candidateLabel(candidate) {
  if (candidate.description) {
    return candidate.description;
  }
  if (candidate.kind === "role") {
    const name = candidate.name instanceof RegExp
      ? candidate.name.toString()
      : JSON.stringify(candidate.name);
    return `role=${candidate.role}[name=${name}]`;
  }
  return `css=${candidate.selector}`;
}

function locatorForCandidate(page, candidate) {
  if (candidate.kind === "role") {
    return page.getByRole(candidate.role, {
      name: candidate.name,
      exact: candidate.exact ?? false,
      includeHidden: candidate.includeHidden ?? false,
    });
  }
  return page.locator(candidate.selector);
}

async function defaultValidation(locator) {
  return {
    valid: true,
    tagName: await locator.evaluate((node) => node.tagName.toLowerCase()),
  };
}

async function evaluateCandidate(page, candidate, validate, requireVisible) {
  const locator = locatorForCandidate(page, candidate);
  const matches = await locator.all();
  const observations = [];
  let match = null;

  for (const candidateLocator of matches) {
    const visible = await candidateLocator.isVisible().catch(() => false);
    if (requireVisible && !visible) {
      observations.push({ visible, valid: false, reason: "not visible" });
      continue;
    }

    try {
      const validation = await (validate || defaultValidation)(candidateLocator, page);
      observations.push({ visible, ...validation });
      if (!match && validation?.valid !== false) {
        match = candidateLocator;
      }
    } catch (error) {
      observations.push({
        visible,
        valid: false,
        reason: safeText(error?.message || error),
      });
    }
  }

  return {
    locator: match,
    observation: {
      candidate: candidateLabel(candidate),
      count: matches.length,
      observations,
    },
  };
}

function createProviderSupport(config) {
  const targetDefinitions = config.targets || {};

  function matchesUrl(value) {
    return config.urlPatterns.some((pattern) => pattern.test(value));
  }

  function findPage(context) {
    return context.pages().find((page) => matchesUrl(page.url())) ?? null;
  }

  async function openPage(context) {
    const page = await context.newPage();
    await page.goto(config.url, {
      waitUntil: "domcontentloaded",
      timeout: 45_000,
    });
    return page;
  }

  async function findSemanticTarget(page, targetName, options = {}) {
    const definition = targetDefinitions[targetName];
    if (!definition) {
      throw new Error(
        `${config.displayName} does not define semantic target ${JSON.stringify(targetName)}.`
      );
    }

    const requireVisible = options.requireVisible ?? definition.requireVisible ?? true;
    const report = {
      provider: config.id,
      target: targetName,
      url: safeUrl(page.url()),
      title: safeText(await page.title().catch(() => ""), 200),
      requireVisible,
      tried: [],
      matchedSelector: null,
      matchedCandidateIndex: null,
    };

    for (const [index, candidate] of definition.candidates.entries()) {
      const result = await evaluateCandidate(
        page,
        candidate,
        definition.validate,
        requireVisible
      );
      report.tried.push(result.observation);
      if (result.locator) {
        report.matchedSelector = result.observation.candidate;
        report.matchedCandidateIndex = index;
        return {
          matched: true,
          locator: result.locator,
          report,
          failureMessage: null,
        };
      }
    }

    return {
      matched: false,
      locator: null,
      report,
      failureMessage:
        `${config.displayName} semantic target ${JSON.stringify(targetName)} did not resolve. ` +
        `Tried: ${report.tried.map((item) => item.candidate).join(", ")}.`,
    };
  }

  async function firstStateMatch(page, stateName) {
    if (!targetDefinitions[stateName]) {
      return null;
    }
    const result = await findSemanticTarget(page, stateName);
    return result.matched ? result : null;
  }

  async function classifyPageState(page) {
    const base = {
      provider: config.id,
      url: safeUrl(page.url()),
      title: safeText(await page.title().catch(() => ""), 200),
      matchedSelectors: [],
      notes: [],
    };

    if (!matchesUrl(page.url())) {
      return {
        ...base,
        kind: "not_found",
        notes: ["The page URL is outside this provider's supported origins."],
      };
    }

    const stateTargets = [
      ["challenge", "challenge"],
      ["permissionRequired", "permission_required"],
      ["loginRequired", "login_required"],
      ["rateLimited", "rate_limited"],
      ["blocked", "blocked"],
      ["notFound", "not_found"],
      ["error", "error"],
    ];

    for (const [targetName, kind] of stateTargets) {
      const matched = await firstStateMatch(page, targetName);
      if (matched) {
        return {
          ...base,
          kind,
          matchedSelectors: [matched.report.matchedSelector],
          selectorReport: matched.report,
        };
      }
    }

    const composer = await findSemanticTarget(page, "composer");
    if (composer.matched) {
      return {
        ...base,
        kind: "ready",
        matchedSelectors: [composer.report.matchedSelector],
        selectorReport: composer.report,
      };
    }

    const loading = await firstStateMatch(page, "loading");
    if (loading) {
      return {
        ...base,
        kind: "loading",
        matchedSelectors: [loading.report.matchedSelector],
        selectorReport: loading.report,
      };
    }

    return {
      ...base,
      kind: "unknown",
      notes: [
        "No known ready, loading, login, permission, challenge, rate-limit, blocked, not-found, or error target matched.",
      ],
      selectorReport: composer.report,
    };
  }

  async function attachJson(testInfo, name, value) {
    await testInfo.attach(name, {
      body: Buffer.from(JSON.stringify(value, null, 2)),
      contentType: "application/json",
    });
  }

  async function assertReadyCanary(page, testInfo, expect) {
    const state = await classifyPageState(page);
    await attachJson(testInfo, `${config.id}-page-state.json`, state);

    expect(
      state.kind,
      `${config.displayName} must be authenticated and ready; classified ${state.kind}.`
    ).toBe("ready");

    const resolved = {};
    for (const targetName of ["composer", "transcript"]) {
      const result = await findSemanticTarget(page, targetName, {
        requireVisible: true,
      });
      await attachJson(
        testInfo,
        `${config.id}-selector-${targetName}.json`,
        result.report
      );
      expect(result.matched, result.failureMessage).toBeTruthy();
      resolved[targetName] = result.locator;
    }

    const composer = resolved.composer;
    const initialDraft = await readComposerText(composer);
    expect(
      initialDraft,
      `${config.displayName} selector canary will not overwrite an existing provider draft.`
    ).toBe("");

    const probe = `CHATMUX_SELECTOR_${crypto.randomUUID().replaceAll("-", "").slice(0, 12)}`;
    let sendResult = null;
    try {
      await composer.fill(probe);
      await expect.poll(() => readComposerText(composer)).toBe(probe);

      sendResult = await findSemanticTarget(page, "sendButton", {
        requireVisible: true,
      });
      await attachJson(
        testInfo,
        `${config.id}-selector-sendButton.json`,
        sendResult.report
      );
      expect(sendResult.matched, sendResult.failureMessage).toBeTruthy();

      const relation = await validateSharedComposerSurface(
        page,
        composer,
        sendResult.locator
      );
      await attachJson(
        testInfo,
        `${config.id}-composer-send-relation.json`,
        relation
      );
      expect(
        relation.sameSurface,
        `${config.displayName} send action did not resolve in the same semantic composer surface.`
      ).toBeTruthy();
    } finally {
      if (config.clearComposer) {
        await config.clearComposer(composer, page);
      } else {
        await composer.fill("");
      }
      await expect
        .poll(() => readComposerText(composer), {
          message: `${config.displayName} selector probe cleanup`,
        })
        .toBe("");
    }
  }

  return {
    ...config,
    attachJson,
    assertReadyCanary,
    classifyPageState,
    findPage,
    findSemanticTarget,
    matchesUrl,
    normalizeText,
    openPage,
    safeText,
    safeUrl,
  };
}

async function readComposerText(locator) {
  return normalizeText(
    await locator.evaluate((node) => {
      if ("value" in node) {
        return node.value || "";
      }
      return node.innerText || node.textContent || "";
    })
  );
}

async function validateSharedComposerSurface(page, composer, sendButton) {
  const composerHandle = await composer.elementHandle();
  const sendHandle = await sendButton.elementHandle();
  if (!composerHandle || !sendHandle) {
    return { sameSurface: false, reason: "element handle unavailable" };
  }

  return page.evaluate(
    ({ composerNode, sendNode }) => {
      const form = composerNode.closest("form");
      if (form) {
        return {
          sameSurface: form.contains(sendNode),
          relation: "shared form",
          commonTag: "form",
        };
      }

      const ancestors = new Map();
      let current = composerNode;
      let distance = 0;
      while (current && distance <= 8) {
        ancestors.set(current, distance);
        current = current.parentElement;
        distance += 1;
      }

      current = sendNode;
      distance = 0;
      while (current && distance <= 8) {
        if (ancestors.has(current)) {
          const tag = current.tagName.toLowerCase();
          return {
            sameSurface: !["html", "body", "main"].includes(tag),
            relation: "near common ancestor",
            commonTag: tag,
            composerDistance: ancestors.get(current),
            sendDistance: distance,
          };
        }
        current = current.parentElement;
        distance += 1;
      }

      return { sameSurface: false, reason: "no near common ancestor" };
    },
    { composerNode: composerHandle, sendNode: sendHandle }
  );
}

async function validateEditable(locator) {
  const facts = await locator.evaluate((node) => ({
    tagName: node.tagName.toLowerCase(),
    contentEditable: node.getAttribute("contenteditable"),
    disabled: "disabled" in node ? Boolean(node.disabled) : false,
    readOnly: "readOnly" in node ? Boolean(node.readOnly) : false,
  }));
  const editableTag = ["textarea", "input"].includes(facts.tagName);
  const contentEditable = facts.contentEditable === "true";
  return {
    valid: (editableTag || contentEditable) && !facts.disabled && !facts.readOnly,
    ...facts,
  };
}

async function validateButton(locator) {
  const facts = await locator.evaluate((node) => ({
    tagName: node.tagName.toLowerCase(),
    type: node.getAttribute("type"),
    ariaLabel: node.getAttribute("aria-label"),
  }));
  return {
    valid: facts.tagName === "button",
    ...facts,
  };
}

async function validateTranscript(locator, page) {
  const facts = await locator.evaluate((node) => ({
    tagName: node.tagName.toLowerCase(),
    role: node.getAttribute("role"),
  }));
  return {
    valid: facts.tagName === "main" || facts.role === "main",
    ...facts,
    pageUrl: safeUrl(page.url()),
  };
}

function role(roleName, name, options = {}) {
  return {
    kind: "role",
    role: roleName,
    name,
    ...options,
  };
}

function css(selector, description) {
  return { kind: "css", selector, description };
}

module.exports = {
  createProviderSupport,
  css,
  normalizeText,
  role,
  safeText,
  safeUrl,
  validateButton,
  validateEditable,
  validateTranscript,
};

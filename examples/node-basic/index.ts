import { createFluxgate } from '../../js/index.js';

async function main() {
  const gate = await createFluxgate({
    policies: [
      {
        id: 'ip-global',
        match: 'ip:*',
        limitPerSecond: 5,
        burst: 5,
        windowSeconds: 60,
      },
    ],
  });

  const decision = gate.check({ ip: '203.0.113.8' });
  console.log('allowed', decision.allowed, 'retryAfter', decision.retryAfterMs);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

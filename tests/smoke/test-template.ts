// tests/smoke/test-template.ts
import '../../scripts/kit-sdk';

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

async function runTests() {
  // Test 1: Basic template with tabstops
  const testName = 'template-basic-tabstops';
  logTest(testName, 'running');
  const start = Date.now();
  
  try {
    console.error('[SMOKE] Starting template test...');
    
    const result = await template(`Hello \${1:world}!

Dear \${2:name},

Please meet me at \${3:address}.

Sincerely,
\${4:signature}`);
    
    logTest(testName, 'pass', { 
      result: result.substring(0, 100),
      duration_ms: Date.now() - start 
    });
  } catch (err) {
    logTest(testName, 'fail', { 
      error: String(err), 
      duration_ms: Date.now() - start 
    });
  }
  
  console.error('[SMOKE] Template tests complete');
  process.exit(0);
}

runTests();

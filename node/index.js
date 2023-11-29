const qif2json = require("qif2json");
const { performance } = require('perf_hooks');

const count = 10000;

const content = "!Type:Bank" + `D02/10/2020
C*
Mtest order 1
T-100.00
PAmazon.com
LFood:Groceries
SFood:Groceries
E50%
$-50.00
STransportation:Automobile
E25%
$-25.00
SPersonal Care:Haircare
E10%
$-10.00
SHealthcare:Prescriptions
E15%
$-15.00
^
`.repeat(count);

const before = performance.now();
const parsed = qif2json.parse(content, { dateFormat: 'MM/DD/YYYY'})
const elapsed = performance.now() - before;

console.log(`NODE: Done processing ${parsed.transactions.length} items. Time it would take to process 1M items: ${(elapsed * 1_000_000 / count).toFixed()}ms`);
import { expect } from 'chai';
import SDK from "@zeitgeistpm/sdk";

describe('Blocks', function() {
    it('block number should not be zero', async function(done) {
        this.timeout(5000);

        const sdk = await SDK.initialize("ws://127.0.0.1:9944");
        const blockHash = await sdk.api.rpc.chain.getFinalizedHead();
        const block = await sdk.api.rpc.chain.getBlock(blockHash);
        expect(block.block.header.number).to.be.gt(0);

        setTimeout(done, 100);
    });
});

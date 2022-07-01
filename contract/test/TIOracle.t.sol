// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/TIOracle.sol";

contract ContractTest is Test {
    TIOracle tiOracle;
    address nodeA;
    address nodeB;
    address nodeC;
    function setUp() public {
        tiOracle = new TIOracle("eth", 5, 300); //for ETH, 5 times each round, max delay 300 seconds
        nodeA = address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266);
        nodeB = address(0x70997970C51812dc3A010C7d01b50e0d17dc79C8);
        nodeC = address(0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC);
    }

    function testRoundOwner() public {
        tiOracle.addNode(nodeA);
        tiOracle.addNode(nodeB);
        tiOracle.addNode(nodeC);
        for(uint i=0;i<15;i++) {
            address node = tiOracle.decideValidNode(i);
            //emit log_named_address("node",node);
            if(i%3 == 0) {
                assertEq(node, nodeA);
            }
            if(i%3 == 1) {
                assertEq(node, nodeB);
            }
            if(i%3 == 2) {
                assertEq(node, nodeC);
            }
        }
    }

    function testRemoveNode() public {
        testRoundOwner();
        tiOracle.removeNode(nodeB);
        for(uint i=0;i<12;i++) {
            address node = tiOracle.decideValidNode(i);
            //emit log_named_address("node",node);
            if(i%2 == 0) {
                assertEq(node, nodeA);
            }
            if(i%2 == 1) {
                assertEq(node, nodeC);
            }
        }
    }

    function fakeEthFeeds() internal view returns (TIOracle.PeerPriceFeed[] memory) {
            TIOracle.PeerPriceFeed[] memory feeds = new TIOracle.PeerPriceFeed[](3);
            TIOracle.PeerPriceFeed memory item;
            item.peerAddress = nodeA;
            item.price = 23456;
            item.timestamp = 1656587035;
            item.sig = hex'764b3b307faabd37ec72270b31b71af012a0f21528e5a581b7b9052a7edc023c69104ef5104ab0dec78f9ddd2a22429cf3e9a44255e3ffd78e70284bed75a8731b';
            TIOracle.PeerPriceFeed memory item2;
            item2.peerAddress = nodeB;
            item2.price = 23457;
            item2.timestamp = 1656587035;
            item2.sig = hex'707b11750fabff7e2fb731e82fa69aa373118f2c41c711892e3284676a3b1af427d7a9423b8fbcb28963cee5d6a4e3c9ef87cb0b6fbe9a462dc096471228a0f41b';
            TIOracle.PeerPriceFeed memory item3;
            item3.peerAddress = nodeC;
            item3.price = 23458;
            item3.timestamp = 1656587035;
            item3.sig = hex'64ae025ad2a1f1d7207407b7df9c61bb859b58daf9f3b397f791ee02d12602a12e933fbf4db53c317225df2f430927d6bc0ffa7d3d8525a070c4112ae87673681c';
            feeds[0] = item;
            feeds[1] = item2;
            feeds[2] = item3;
            return feeds;
    }

    function fakeBTCFeeds() internal view returns (TIOracle.PeerPriceFeed[] memory) {
            TIOracle.PeerPriceFeed[] memory feeds = new TIOracle.PeerPriceFeed[](3);
            TIOracle.PeerPriceFeed memory item;
            item.peerAddress = nodeA;
            item.price = 23456;
            item.timestamp = 1656587035;
            item.sig = hex'85991170e857be37efcfdbe57aacc27fe901465e1b1f71b3fe6fb2290e7ff001121c0ca78ded21dabd6cff104e3fe0bebed75ba9be8417144067eee3f64bb1a91b';
            TIOracle.PeerPriceFeed memory item2;
            item2.peerAddress = nodeB;
            item2.price = 23457;
            item2.timestamp = 1656587035;
            item2.sig = hex'7271e8f1fe901e4f6707656e9be96e3acb958f4c3bd9c103719afa19ebf53862431c6d3f459ae91060a7e9b3d78c7ca2ef9d3dd03b95b1f1635012741c7617131c';
            TIOracle.PeerPriceFeed memory item3;
            item3.peerAddress = nodeC;
            item3.price = 23458;
            item3.timestamp = 1656587035;
            item3.sig = hex'625801a969ef917ca023116b57d86aac67a82df84b1a8e1535bcf223244c4e475e0c6d4d7247713cd6e9ffb4c463d300d29223895d9fa17072dce05fdce829061c';
            feeds[0] = item;
            feeds[1] = item2;
            feeds[2] = item3;
            return feeds;
    }

    function testFeedPrice() public {
        Vm vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
        testRoundOwner();
        vm.startPrank(nodeA);
        TIOracle.PeerPriceFeed[] memory btcFeeds = fakeBTCFeeds();
        TIOracle.PeerPriceFeed[] memory ethFeeds = fakeEthFeeds();
        tiOracle.feedPrice("eth", ethFeeds);
        tiOracle.feedPrice("eth", ethFeeds);
        tiOracle.feedPrice("eth", ethFeeds);
        tiOracle.feedPrice("eth", ethFeeds);
        tiOracle.feedPrice("eth", ethFeeds);
        assertTrue(!tiOracle.isMyTurn());
        //nodeA can only feed 5 times, and then it is nodeB's turn now
        //Here, we cheat to make it timeout, so that any nodes could feed
        vm.warp(block.timestamp + 301);
        assertTrue(tiOracle.isMyTurn());
        vm.stopPrank();

        vm.startPrank(nodeB);
        assertTrue(tiOracle.isMyTurn());
        tiOracle.feedPrice("eth", ethFeeds);
        vm.expectRevert("coin mismatch");
        tiOracle.feedPrice("btc", btcFeeds); //unexpected coin
        vm.stopPrank();
        assertEq(tiOracle.queryPrice().price, 23457); //median
    }

    function testKickNode() public {
        Vm vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
        testRoundOwner();
        vm.prank(nodeA);
        tiOracle.kickNode(nodeC);
        vm.prank(nodeB);
        tiOracle.kickNode(nodeC);
        vm.prank(nodeC);
        tiOracle.kickNode(nodeC);
    }

    function testVerify() public {
        bytes memory  digest = hex'6865656c6c20776f726c640000000000000000000000000000000000000000000000000000000000003039';
        bytes memory sign = hex'3571db3a6e9027358a0acd06a3596a4ba6307adde571ca97fb2416b208ec1ac015e2d8538a2ff270f10b253034ab1caffcd0e367ca7c3991c79ab4acc7af14321c';
        address rec = tiOracle.recoverSign(keccak256(digest), sign);
        emit log_named_address("recover", rec) ;
    }
}

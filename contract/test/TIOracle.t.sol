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
        tiOracle = new TIOracle(5, 300); //5 feed each round, max delay 300 seconds
        nodeA = address(0x5A);
        nodeB = address(0x5B);
        nodeC = address(0x5C);
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

    function testFeedPrice() public {
        Vm vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
        testRoundOwner();
        vm.startPrank(nodeA);
        tiOracle.feedPrice("btc", 123);
        tiOracle.feedPrice("eth", 12345);
        tiOracle.feedPrice("eth", 12345);
        tiOracle.feedPrice("eth", 12345);
        tiOracle.feedPrice("eth", 12345);
        assertTrue(!tiOracle.isMyTurn());
        vm.stopPrank();
        //nodeA can only feed 5 times, and then it is nodeB's turn now
        vm.startPrank(nodeB);
        assertTrue(tiOracle.isMyTurn());
        tiOracle.feedPrice("eth", 1234567);
        vm.stopPrank();
        TIOracle.PriceInfo memory ethPrice = tiOracle.queryPrice("eth");
        assertEq(ethPrice.tiPrice, 1234567);
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

}

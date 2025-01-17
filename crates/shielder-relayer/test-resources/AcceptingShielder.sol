// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

contract AcceptingShielder {
    function withdrawNative(
        bytes3 expectedContractVersion,
        uint256 idHiding,
        uint256 amount,
        address withdrawAddress,
        uint256 merkleRoot,
        uint256 nullifier,
        uint256 newNote,
        bytes calldata proof,
        address relayer,
        uint256 relayerFee
    ) external {}
}

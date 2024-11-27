import { NewAccountAction, NewAccountCalldata } from "./newAccount";
import { DepositAction, DepositCalldata } from "./deposit";
import { WithdrawAction, WithdrawCalldata } from "./withdraw";

export interface Calldata {
  provingTimeMillis: number;
}

export {
  DepositAction,
  DepositCalldata,
  NewAccountAction,
  NewAccountCalldata,
  WithdrawAction,
  WithdrawCalldata
};

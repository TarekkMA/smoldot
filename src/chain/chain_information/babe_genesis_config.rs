// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    executor::{host, vm},
    header,
};

use alloc::vec::Vec;
use core::num::NonZeroU64;

/// BABE configuration of a chain, as extracted from the genesis block.
///
/// The way a chain configures BABE is stored in its runtime.
#[derive(Debug, Clone)]
pub struct BabeGenesisConfiguration {
    pub slots_per_epoch: NonZeroU64,
    pub epoch0_configuration: header::BabeNextConfig,
    pub epoch0_information: header::BabeNextEpoch,
}

impl BabeGenesisConfiguration {
    /// Retrieves the configuration from the given virtual machine prototype.
    ///
    /// Must be passed a closure that returns the storage value corresponding to the given key in
    /// the genesis block storage.
    ///
    /// Returns back the same virtual machine prototype as was passed as parameter.
    pub fn from_virtual_machine_prototype(
        vm: host::HostVmPrototype,
        mut genesis_storage_access: impl FnMut(&[u8]) -> Option<Vec<u8>>,
    ) -> (Result<Self, FromVmPrototypeError>, host::HostVmPrototype) {
        let mut vm: host::HostVm = match vm.run_no_param("BabeApi_configuration") {
            Ok(vm) => vm.into(),
            Err((err, proto)) => return (Err(FromVmPrototypeError::VmStart(err)), proto),
        };

        loop {
            match vm {
                host::HostVm::ReadyToRun(r) => vm = r.run(),
                host::HostVm::Finished(finished) => {
                    let cfg = {
                        let output = finished.value();
                        let val = match nom::combinator::all_consuming(decode_genesis_config)(
                            output.as_ref(),
                        ) {
                            Ok((_, parse_result)) => Ok(parse_result),
                            Err(_) => Err(FromVmPrototypeError::OutputDecode),
                        };
                        // Note: this is a bit convoluted, but I have no idea how to satisfy the
                        // borrow checker other than by doing so.
                        drop(output);
                        val
                    };

                    break (cfg, finished.into_prototype());
                }
                host::HostVm::Error { prototype, .. } => {
                    break (Err(FromVmPrototypeError::Trapped), prototype)
                }

                host::HostVm::ExternalStorageGet(req) => {
                    let value = genesis_storage_access(req.key().as_ref());
                    vm = req.resume_full_value(value.as_ref().map(|v| &v[..]));
                }

                host::HostVm::GetMaxLogLevel(resume) => {
                    vm = resume.resume(0); // Off
                }
                host::HostVm::LogEmit(req) => vm = req.resume(),

                other => {
                    let prototype = other.into_prototype();
                    break (Err(FromVmPrototypeError::HostFunctionNotAllowed), prototype);
                }
            }
        }
    }
}

/// Error when retrieving the BABE configuration.
#[derive(Debug, derive_more::Display)]
pub enum FromVmPrototypeError {
    /// Error when starting the virtual machine.
    #[display(fmt = "{}", _0)]
    VmStart(host::StartErr),
    /// Crash while running the virtual machine.
    Trapped,
    /// Virtual machine tried to call a host function that isn't valid in this context.
    HostFunctionNotAllowed,
    /// Error while decoding the output of the virtual machine.
    OutputDecode,
}

impl FromVmPrototypeError {
    /// Returns `true` if this error is about an invalid function.
    pub fn is_function_not_found(&self) -> bool {
        matches!(
            self,
            FromVmPrototypeError::VmStart(host::StartErr::VirtualMachine(
                vm::StartErr::FunctionNotFound | vm::StartErr::NotAFunction
            ))
        )
    }
}

fn decode_genesis_config(bytes: &[u8]) -> nom::IResult<&[u8], BabeGenesisConfiguration> {
    nom::combinator::map(
        nom::sequence::tuple((
            nom::number::complete::le_u64,
            nom::combinator::map_opt(nom::number::complete::le_u64, NonZeroU64::new),
            nom::number::complete::le_u64,
            nom::number::complete::le_u64,
            nom::combinator::flat_map(crate::util::nom_scale_compact_usize, |num_elems| {
                nom::multi::many_m_n(
                    num_elems,
                    num_elems,
                    nom::combinator::map(
                        nom::sequence::tuple((
                            nom::bytes::complete::take(32u32),
                            nom::number::complete::le_u64,
                        )),
                        move |(public_key, weight)| header::BabeAuthority {
                            public_key: <[u8; 32]>::try_from(public_key).unwrap(),
                            weight,
                        },
                    ),
                )
            }),
            nom::combinator::map(nom::bytes::complete::take(32u32), |b| {
                <[u8; 32]>::try_from(b).unwrap()
            }),
            nom::branch::alt((
                nom::combinator::map(nom::bytes::complete::tag(&[0]), |_| {
                    header::BabeAllowedSlots::PrimarySlots
                }),
                nom::combinator::map(nom::bytes::complete::tag(&[1]), |_| {
                    header::BabeAllowedSlots::PrimaryAndSecondaryPlainSlots
                }),
                nom::combinator::map(nom::bytes::complete::tag(&[2]), |_| {
                    header::BabeAllowedSlots::PrimaryAndSecondaryVrfSlots
                }),
            )),
        )),
        |(_slot_duration, slots_per_epoch, c0, c1, authorities, randomness, allowed_slots)| {
            // Note that the slot duration is unused as it is not modifiable anyway.
            BabeGenesisConfiguration {
                slots_per_epoch,
                epoch0_configuration: header::BabeNextConfig {
                    c: (c0, c1),
                    allowed_slots,
                },
                epoch0_information: header::BabeNextEpoch {
                    randomness,
                    authorities,
                },
            }
        },
    )(bytes)
}

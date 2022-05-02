pub(crate) mod v1;

use cf_runtime_upgrade_utilities::VersionedMigration;

pub(crate) type PalletMigration<T> =
	(VersionedMigration<crate::Pallet<T>, v1::Migration<T>, 0, 1>,);

CREATE TABLE `commands` (
	`room_id` text NOT NULL,
	`seat` integer NOT NULL,
	`command_id` text NOT NULL,
	`intent_hash` text NOT NULL,
	`response` text NOT NULL,
	`version` integer NOT NULL,
	PRIMARY KEY(`room_id`, `command_id`),
	FOREIGN KEY (`room_id`) REFERENCES `rooms`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `entry_receipts` (
	`request_hash` text PRIMARY KEY NOT NULL,
	`intent_hash` text NOT NULL,
	`response` text NOT NULL,
	`room_id` text NOT NULL,
	FOREIGN KEY (`room_id`) REFERENCES `rooms`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `journal` (
	`room_id` text NOT NULL,
	`version` integer NOT NULL,
	`entry` text NOT NULL,
	PRIMARY KEY(`room_id`, `version`),
	FOREIGN KEY (`room_id`) REFERENCES `rooms`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `rooms` (
	`id` text PRIMARY KEY NOT NULL,
	`invite` text NOT NULL,
	`initial_state` text NOT NULL,
	`state` text NOT NULL,
	`version` integer NOT NULL,
	`attempt_nonce` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `rooms_invite_unique` ON `rooms` (`invite`);--> statement-breakpoint
CREATE TABLE `seats` (
	`room_id` text NOT NULL,
	`seat` integer NOT NULL,
	`token_hash` text NOT NULL,
	PRIMARY KEY(`room_id`, `seat`),
	FOREIGN KEY (`room_id`) REFERENCES `rooms`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE UNIQUE INDEX `seats_token` ON `seats` (`room_id`,`token_hash`);
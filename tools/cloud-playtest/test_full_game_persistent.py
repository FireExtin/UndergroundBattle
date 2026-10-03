"""Default unittest discovery covers QA-runner failure and cancellation cleanup."""
import asyncio
import json
import tempfile
import unittest
from pathlib import Path

from full_game_persistent import FullGame


class CancellationTests(unittest.IsolatedAsyncioTestCase):
    async def test_cancelled_run_records_failure_and_closes_owned_browser(self):
        class Context:
            closed = False

            async def close(self):
                self.closed = True

        class PausedRun(FullGame):
            async def new_seat(self, seat, session=None):
                self.contexts.append(context)
                self.pages.append(object())
                started.set()
                await asyncio.Future()

            async def view(self, seat):
                return None

        context = Context()
        started = asyncio.Event()
        service = type('Remote', (), {'remote': True, 'base_url': 'https://qa.invalid'})()
        with tempfile.TemporaryDirectory() as folder:
            run = PausedRun(None, service, Path(folder), 'teams', 10, True, False, True)
            task = asyncio.create_task(run.play_full())
            await started.wait()
            task.cancel()
            with self.assertRaises(asyncio.CancelledError):
                await task
            result = json.loads((Path(folder) / 'full-game-summary.json').read_text())
            self.assertFalse(result['passed'])
            self.assertTrue(result['cancelled'])
            self.assertEqual(result['uiPostCount'], 0)
            self.assertTrue(context.closed)

    async def test_unavailable_final_page_still_preserves_cancellation_and_cleanup(self):
        class Context:
            closed = False

            async def close(self):
                self.closed = True

        class PausedRun(FullGame):
            async def new_seat(self, seat, session=None):
                self.contexts.append(context)
                self.pages.append(object())
                started.set()
                await asyncio.Future()

            async def view(self, seat):
                raise RuntimeError('Page was already closed')

        context = Context()
        started = asyncio.Event()
        service = type('Local', (), {'base_url': 'http://qa.invalid'})()
        with tempfile.TemporaryDirectory() as folder:
            run = PausedRun(None, service, Path(folder), 'teams', 10, True, False, True)
            task = asyncio.create_task(run.play_full())
            await started.wait()
            task.cancel()
            with self.assertRaises(asyncio.CancelledError):
                await task
            result = json.loads((Path(folder) / 'full-game-summary.json').read_text())
            self.assertFalse(result['passed'])
            self.assertTrue(result['cancelled'])
            self.assertEqual(result['finalObservationFailure'], 'RuntimeError')
            self.assertIsNone(result['final'])
            self.assertTrue(context.closed)


if __name__ == '__main__':
    unittest.main()

package com.lengsuo.mobile.ui.shell

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.lengsuo.mobile.R
import com.lengsuo.mobile.ui.discover.DiscoverPage
import com.lengsuo.mobile.ui.home.TasksHomePage
import com.lengsuo.mobile.ui.me.MePage
import com.lengsuo.mobile.ui.scheduled.ScheduledPage

@Composable
fun MainShell() {
    var selected by remember { mutableStateOf(MainTab.TASKS) }
    Scaffold(
        bottomBar = {
            NavigationBar {
                MainTab.entries.forEach { tab ->
                    NavigationBarItem(
                        selected = selected == tab,
                        onClick = { selected = tab },
                        icon = { Icon(tab.icon, contentDescription = stringResource(tab.labelRes)) },
                        label = { Text(stringResource(tab.labelRes)) },
                    )
                }
            }
        },
    ) { padding ->
        Box(
            modifier = Modifier.padding(padding).fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            when (selected) {
                MainTab.ME -> MePage()
                MainTab.TASKS -> TasksHomePage()
                MainTab.SCHEDULED -> ScheduledPage()
                MainTab.DISCOVER -> DiscoverPage()
                else -> TabPlaceholder(stringResource(selected.labelRes))
            }
        }
    }
}

@Composable
private fun TabPlaceholder(label: String) {
    Text(
        text = label + " · " + stringResource(R.string.tab_coming_soon),
        style = MaterialTheme.typography.bodyLarge,
        color = MaterialTheme.colorScheme.onSurface,
    )
}

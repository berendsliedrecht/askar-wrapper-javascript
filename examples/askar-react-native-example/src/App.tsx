import { askar } from '@openwallet-foundation/askar-react-native'
import { Platform, StyleSheet, Text, View } from 'react-native'

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
})

export const App = () => {
  const { major, minor, patch } = Platform.constants.reactNativeVersion

  return (
    <View style={styles.container}>
      <Text>React native version: {`${major}.${minor}.${patch}`}</Text>
      <Text>askar version: {askar.version()}</Text>
    </View>
  )
}
